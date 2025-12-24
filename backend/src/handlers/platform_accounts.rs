use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Datelike};

use crate::models::{git_platform_account, contribution};
use crate::services::git_platforms::{github::GitHubClient, gitea::GiteaClient, GitPlatform, PlatformConfig};
use crate::utils::{config::Config, encryption, validators};

#[derive(Debug, Deserialize)]
pub struct ConnectPlatformRequest {
    pub platform: String,
    pub access_token: String,
    pub instance_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSyncPreferencesRequest {
    pub sync_profile: bool,
    pub sync_contributions: bool,
    pub sync_activities: bool,
}

#[derive(Debug, Serialize)]
pub struct PlatformAccountResponse {
    pub id: String,
    pub platform: String,
    pub platform_username: String,
    pub platform_url: Option<String>,
    pub is_active: bool,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    // Profile fields
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub profile_url: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub followers_count: Option<i32>,
    pub following_count: Option<i32>,
    // Sync preferences
    pub sync_profile: bool,
    pub sync_contributions: bool,
    pub sync_activities: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// POST /api/platforms/connect
/// Connect a git platform account using a personal access token
pub async fn connect_platform(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<ConnectPlatformRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Validate platform
    validators::validate_platform(&payload.platform).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid platform: {}", e))
    })?;

    // Determine platform type and configuration
    let (platform_type, platform_config) = match payload.platform.as_str() {
        "github" => {
            if payload.instance_url.is_some() {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: "GitHub does not support custom instances".to_string(),
                }));
            }
            (git_platform_account::GitPlatform::GitHub, PlatformConfig::github())
        }
        "gitea" => {
            let instance_url = payload.instance_url.as_ref()
                .ok_or_else(|| {
                    actix_web::error::ErrorBadRequest("Gitea requires an instance URL")
                })?;

            // Validate instance URL
            validators::validate_url(instance_url).map_err(|e| {
                actix_web::error::ErrorBadRequest(format!("Invalid instance URL: {}", e))
            })?;

            (git_platform_account::GitPlatform::Gitea, PlatformConfig::gitea_custom(instance_url))
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Unsupported platform: {}", payload.platform),
            }));
        }
    };

    // Validate the access token and get user info
    let user_info = match payload.platform.as_str() {
        "github" => {
            let client = GitHubClient::new();
            client
                .validate_token(&platform_config, &payload.access_token)
                .await
                .map_err(|e| {
                    log::error!("Failed to validate GitHub token: {}", e);
                    actix_web::error::ErrorUnauthorized(format!("Invalid access token: {}", e))
                })?
        }
        "gitea" => {
            let client = GiteaClient::new();
            client
                .validate_token(&platform_config, &payload.access_token)
                .await
                .map_err(|e| {
                    log::error!("Failed to validate Gitea token: {}", e);
                    actix_web::error::ErrorUnauthorized(format!("Invalid access token: {}", e))
                })?
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Unsupported platform: {}", payload.platform),
            }));
        }
    };

    // Encrypt the access token
    let encrypted_token = encryption::encrypt(&payload.access_token, &config.encryption_key)
        .map_err(|e| {
            log::error!("Failed to encrypt token: {}", e);
            actix_web::error::ErrorInternalServerError("Encryption failed")
        })?;

    // Check if this platform account already exists
    let existing_account = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::PlatformType.eq(platform_type.clone()))
        .filter(git_platform_account::Column::PlatformUsername.eq(&user_info.username))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let account = if let Some(account) = existing_account {
        // Update existing account
        let mut account: git_platform_account::ActiveModel = account.into();
        account.access_token = Set(Some(encrypted_token));
        account.is_active = Set(true);
        account.updated_at = Set(chrono::Utc::now());

        account.update(db.as_ref()).await.map_err(|e| {
            log::error!("Failed to update account: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update account")
        })?
    } else {
        // Create new account
        let new_account = git_platform_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            platform_type: Set(platform_type),
            platform_username: Set(user_info.username.clone()),
            access_token: Set(Some(encrypted_token)),
            refresh_token: Set(None),
            platform_url: Set(payload.instance_url.clone()),
            is_active: Set(true),
            last_synced_at: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            avatar_url: Set(user_info.avatar_url),
            display_name: Set(None),
            bio: Set(None),
            profile_url: Set(None),
            location: Set(None),
            company: Set(None),
            followers_count: Set(None),
            following_count: Set(None),
            sync_profile: Set(true),
            sync_contributions: Set(true),
            sync_activities: Set(true),
        };

        git_platform_account::Entity::insert(new_account)
            .exec_with_returning(db.as_ref())
            .await
            .map_err(|e| {
                log::error!("Failed to create account: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create account")
            })?
    };

    let platform_str = match account.platform_type {
        git_platform_account::GitPlatform::GitHub => "github",
        git_platform_account::GitPlatform::GitLab => "gitlab",
        git_platform_account::GitPlatform::Gitea => "gitea",
    };

    Ok(HttpResponse::Ok().json(PlatformAccountResponse {
        id: account.id.to_string(),
        platform: platform_str.to_string(),
        platform_username: account.platform_username,
        platform_url: account.platform_url,
        is_active: account.is_active,
        last_synced_at: account.last_synced_at.map(|dt| dt.to_rfc3339()),
        created_at: account.created_at.to_rfc3339(),
        updated_at: account.updated_at.to_rfc3339(),
        avatar_url: account.avatar_url,
        display_name: account.display_name,
        bio: account.bio,
        profile_url: account.profile_url,
        location: account.location,
        company: account.company,
        followers_count: account.followers_count,
        following_count: account.following_count,
        sync_profile: account.sync_profile,
        sync_contributions: account.sync_contributions,
        sync_activities: account.sync_activities,
    }))
}

/// GET /api/platforms
/// List all connected platform accounts for the current user
pub async fn list_platforms(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let accounts = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let response: Vec<PlatformAccountResponse> = accounts
        .into_iter()
        .map(|account| {
            let platform_str = match account.platform_type {
                git_platform_account::GitPlatform::GitHub => "github",
                git_platform_account::GitPlatform::GitLab => "gitlab",
                git_platform_account::GitPlatform::Gitea => "gitea",
            };

            PlatformAccountResponse {
                id: account.id.to_string(),
                platform: platform_str.to_string(),
                platform_username: account.platform_username,
                platform_url: account.platform_url,
                is_active: account.is_active,
                last_synced_at: account.last_synced_at.map(|dt| dt.to_rfc3339()),
                created_at: account.created_at.to_rfc3339(),
                updated_at: account.updated_at.to_rfc3339(),
                avatar_url: account.avatar_url,
                display_name: account.display_name,
                bio: account.bio,
                profile_url: account.profile_url,
                location: account.location,
                company: account.company,
                followers_count: account.followers_count,
                following_count: account.following_count,
                sync_profile: account.sync_profile,
                sync_contributions: account.sync_contributions,
                sync_activities: account.sync_activities,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/platforms/:id
/// Disconnect a platform account
pub async fn disconnect_platform(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let account_id = Uuid::parse_str(&path.into_inner()).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid account ID: {}", e))
    })?;

    // Find the account
    let account = git_platform_account::Entity::find_by_id(account_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

    // Verify ownership
    if account.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    // Mark as inactive instead of deleting (soft delete)
    let mut account: git_platform_account::ActiveModel = account.into();
    account.is_active = Set(false);
    account.updated_at = Set(chrono::Utc::now());

    account.update(db.as_ref()).await.map_err(|e| {
        log::error!("Failed to deactivate account: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to disconnect account")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Platform disconnected successfully"
    })))
}

/// PUT /api/platforms/:id/sync-preferences
/// Update sync preferences for a platform account
pub async fn update_sync_preferences(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    payload: web::Json<UpdateSyncPreferencesRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let account_id = Uuid::parse_str(&path.into_inner()).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid account ID: {}", e))
    })?;

    // Find the account
    let account = git_platform_account::Entity::find_by_id(account_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

    // Verify ownership
    if account.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    // Validate that at least one sync type is enabled
    if !payload.sync_profile && !payload.sync_contributions && !payload.sync_activities {
        return Err(actix_web::error::ErrorBadRequest(
            "At least one sync type must be enabled (Profile, Heatmap, or Activities)"
        ));
    }

    // Update sync preferences
    let mut account: git_platform_account::ActiveModel = account.into();
    account.sync_profile = Set(payload.sync_profile);
    account.sync_contributions = Set(payload.sync_contributions);
    account.sync_activities = Set(payload.sync_activities);
    account.updated_at = Set(chrono::Utc::now());

    let updated_account = account.update(db.as_ref()).await.map_err(|e| {
        log::error!("Failed to update sync preferences: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update sync preferences")
    })?;

    let platform_str = match updated_account.platform_type {
        git_platform_account::GitPlatform::GitHub => "github",
        git_platform_account::GitPlatform::GitLab => "gitlab",
        git_platform_account::GitPlatform::Gitea => "gitea",
    };

    Ok(HttpResponse::Ok().json(PlatformAccountResponse {
        id: updated_account.id.to_string(),
        platform: platform_str.to_string(),
        platform_username: updated_account.platform_username,
        platform_url: updated_account.platform_url,
        is_active: updated_account.is_active,
        last_synced_at: updated_account.last_synced_at.map(|dt| dt.to_rfc3339()),
        created_at: updated_account.created_at.to_rfc3339(),
        updated_at: updated_account.updated_at.to_rfc3339(),
        avatar_url: updated_account.avatar_url,
        display_name: updated_account.display_name,
        bio: updated_account.bio,
        profile_url: updated_account.profile_url,
        location: updated_account.location,
        company: updated_account.company,
        followers_count: updated_account.followers_count,
        following_count: updated_account.following_count,
        sync_profile: updated_account.sync_profile,
        sync_contributions: updated_account.sync_contributions,
        sync_activities: updated_account.sync_activities,
    }))
}

/// POST /api/platforms/:id/sync?all_years=true
/// Manually trigger a sync for a platform account
pub async fn sync_platform(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let account_id = Uuid::parse_str(&path.into_inner()).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid account ID: {}", e))
    })?;

    // Find the account
    let account = git_platform_account::Entity::find_by_id(account_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

    // Verify ownership
    if account.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    if !account.is_active {
        return Err(actix_web::error::ErrorBadRequest("Account is not active"));
    }

    // Get the access token
    let encrypted_token = account.access_token.clone()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No access token found"))?;

    // Decrypt the access token
    let access_token = encryption::decrypt(&encrypted_token, &config.encryption_key)
        .map_err(|e| {
            log::error!("Failed to decrypt token: {}", e);
            actix_web::error::ErrorInternalServerError("Decryption failed")
        })?;

    log::info!("🔄 [Sync] Starting contribution sync for account: {}", account_id);
    log::info!("🔄 [Sync] Platform: {:?}, Username: {}", account.platform_type, account.platform_username);

    // Check sync mode: all_years, specific year, or current year (default)
    let sync_all_years = query.get("all_years")
        .map(|v| v == "true")
        .unwrap_or(false);

    let specific_year = query.get("year")
        .and_then(|v| v.parse::<i32>().ok());

    // Fetch contributions from GitHub
    match account.platform_type {
        git_platform_account::GitPlatform::GitHub => {
            let github_client = GitHubClient::new();
            let platform_config = PlatformConfig::github();

            let current_year = Utc::now().year();

            let (start_year, end_year) = if sync_all_years {
                log::info!("🔄 [Sync] Mode: ALL YEARS (2020 to {})", current_year);
                (2020, current_year)
            } else if let Some(year) = specific_year {
                log::info!("🔄 [Sync] Mode: SPECIFIC YEAR ({})", year);
                (year, year)
            } else {
                log::info!("🔄 [Sync] Mode: CURRENT YEAR ({})", current_year);
                (current_year, current_year)
            };

            let mut all_contributions = Vec::new();
            let mut total_inserted = 0;
            let mut total_updated = 0;

            for year in start_year..=end_year {
                let from_date = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let from = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();

                let to_date = if year == current_year {
                    // For current year, fetch up to today
                    Utc::now().date_naive()
                } else {
                    // For past years, fetch the whole year
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
                };
                let to = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

                log::info!("🔄 [Sync] Fetching year {}: {} to {}", year, from.format("%Y-%m-%d"), to.format("%Y-%m-%d"));

                let contributions = github_client
                    .fetch_contributions(&platform_config, &account.platform_username, &access_token, from, to)
                    .await
                    .map_err(|e| {
                        log::error!("❌ [Sync] Failed to fetch contributions for year {}: {}", year, e);
                        actix_web::error::ErrorInternalServerError(format!("Failed to fetch contributions for year {}: {}", year, e))
                    })?;

                log::info!("✅ [Sync] Fetched {} contribution days for year {}", contributions.len(), year);
                all_contributions.extend(contributions);
            }

            log::info!("✅ [Sync] Total fetched: {} contribution days across all years", all_contributions.len());

            // Store contributions in database
            for contrib in all_contributions {
                // Check if contribution already exists for this date AND repository
                let existing = contribution::Entity::find()
                    .filter(contribution::Column::GitPlatformAccountId.eq(account_id))
                    .filter(contribution::Column::ContributionDate.eq(contrib.date))
                    .filter(
                        if let Some(ref repo) = contrib.repository_name {
                            contribution::Column::RepositoryName.eq(repo.as_str())
                        } else {
                            contribution::Column::RepositoryName.is_null()
                        }
                    )
                    .one(db.as_ref())
                    .await
                    .map_err(|e| {
                        log::error!("Database error: {}", e);
                        actix_web::error::ErrorInternalServerError("Database error")
                    })?;

                if let Some(existing_contrib) = existing {
                    // Update existing contribution
                    let mut active_model: contribution::ActiveModel = existing_contrib.into();
                    active_model.count = Set(contrib.count);
                    active_model.updated_at = Set(Utc::now());

                    active_model.update(db.as_ref()).await.map_err(|e| {
                        log::error!("Failed to update contribution: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to update contribution")
                    })?;
                    total_updated += 1;
                } else {
                    // Insert new contribution
                    let new_contrib = contribution::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        git_platform_account_id: Set(account_id),
                        contribution_date: Set(contrib.date),
                        count: Set(contrib.count),
                        repository_name: Set(contrib.repository_name),
                        is_private_repo: Set(contrib.is_private),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                    };

                    contribution::Entity::insert(new_contrib)
                        .exec(db.as_ref())
                        .await
                        .map_err(|e| {
                            log::error!("Failed to insert contribution: {}", e);
                            actix_web::error::ErrorInternalServerError("Failed to insert contribution")
                        })?;
                    total_inserted += 1;
                }
            }

            log::info!("💾 [Sync] Stored contributions: {} inserted, {} updated", total_inserted, total_updated);

            // Fetch and update profile data
            log::info!("👤 [Sync] Fetching profile data for {}", account.platform_username);
            match github_client.fetch_user_profile(&platform_config, &account.platform_username, &access_token).await {
                Ok(profile_data) => {
                    log::info!("✅ [Sync] Fetched profile data successfully");

                    // Extract profile fields from the JSON response
                    let avatar_url = profile_data.get("avatar_url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let display_name = profile_data.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let bio = profile_data.get("bio")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let profile_url = profile_data.get("html_url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let location = profile_data.get("location")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let company = profile_data.get("company")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let followers_count = profile_data.get("followers")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32);

                    let following_count = profile_data.get("following")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32);

                    // Update account with profile data
                    let profile_account = git_platform_account::Entity::find_by_id(account_id)
                        .one(db.as_ref())
                        .await
                        .map_err(|e| {
                            log::error!("Database error: {}", e);
                            actix_web::error::ErrorInternalServerError("Database error")
                        })?
                        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

                    let mut profile_active: git_platform_account::ActiveModel = profile_account.into();
                    profile_active.avatar_url = Set(avatar_url);
                    profile_active.display_name = Set(display_name);
                    profile_active.bio = Set(bio);
                    profile_active.profile_url = Set(profile_url);
                    profile_active.location = Set(location);
                    profile_active.company = Set(company);
                    profile_active.followers_count = Set(followers_count);
                    profile_active.following_count = Set(following_count);
                    profile_active.updated_at = Set(Utc::now());

                    profile_active.update(db.as_ref()).await.map_err(|e| {
                        log::error!("Failed to update profile data: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to update profile")
                    })?;

                    log::info!("💾 [Sync] Stored profile data successfully");
                }
                Err(e) => {
                    log::warn!("⚠️  [Sync] Failed to fetch profile data (continuing sync): {}", e);
                    // Don't fail the entire sync if profile fetch fails
                }
            }
        }
        git_platform_account::GitPlatform::Gitea => {
            let gitea_client = GiteaClient::new();
            let instance_url = account.platform_url.as_ref()
                .ok_or_else(|| actix_web::error::ErrorInternalServerError("Gitea instance URL not found"))?;
            let platform_config = PlatformConfig::gitea_custom(instance_url);

            let current_year = Utc::now().year();

            let (start_year, end_year) = if sync_all_years {
                log::info!("🔄 [Sync] Mode: ALL YEARS (2020 to {})", current_year);
                (2020, current_year)
            } else if let Some(year) = specific_year {
                log::info!("🔄 [Sync] Mode: SPECIFIC YEAR ({})", year);
                (year, year)
            } else {
                log::info!("🔄 [Sync] Mode: CURRENT YEAR ({})", current_year);
                (current_year, current_year)
            };

            // Delete ALL existing Gitea contributions for this account to avoid conflicts
            // This ensures clean data when switching between different sync methods
            let deleted = contribution::Entity::delete_many()
                .filter(contribution::Column::GitPlatformAccountId.eq(account_id))
                .exec(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Failed to delete existing contributions: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to delete existing contributions")
                })?;

            log::info!("🗑️  Deleted {} existing Gitea contributions (fresh sync)", deleted.rows_affected);

            let mut all_contributions = Vec::new();
            let mut total_inserted = 0;
            let mut total_updated = 0;

            for year in start_year..=end_year {
                let from_date = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let from = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();

                let to_date = if year == current_year {
                    Utc::now().date_naive()
                } else {
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
                };
                let to = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

                log::info!("🔄 [Sync] Fetching year {}: {} to {}", year, from.format("%Y-%m-%d"), to.format("%Y-%m-%d"));

                let contributions = gitea_client
                    .fetch_contributions(&platform_config, &account.platform_username, &access_token, from, to)
                    .await
                    .map_err(|e| {
                        log::error!("❌ [Sync] Failed to fetch Gitea contributions for year {}: {}", year, e);
                        actix_web::error::ErrorInternalServerError(format!("Failed to fetch contributions for year {}: {}", year, e))
                    })?;

                log::info!("✅ [Sync] Fetched {} contribution days for year {}", contributions.len(), year);
                all_contributions.extend(contributions);
            }

            log::info!("✅ [Sync] Total fetched: {} contribution days across all years", all_contributions.len());

            // Store contributions in database
            log::info!("💾 [Sync] Starting database storage loop...");
            for (index, contrib) in all_contributions.iter().enumerate() {
                log::debug!("Processing contribution {}/{}: date={}, count={}, repo={:?}",
                    index + 1, all_contributions.len(), contrib.date, contrib.count, contrib.repository_name);

                // Check if contribution already exists for this date AND repository
                let existing = contribution::Entity::find()
                    .filter(contribution::Column::GitPlatformAccountId.eq(account_id))
                    .filter(contribution::Column::ContributionDate.eq(contrib.date))
                    .filter(
                        if let Some(ref repo) = contrib.repository_name {
                            contribution::Column::RepositoryName.eq(repo.as_str())
                        } else {
                            contribution::Column::RepositoryName.is_null()
                        }
                    )
                    .one(db.as_ref())
                    .await
                    .map_err(|e| {
                        log::error!("Database error while checking existing contribution: {}", e);
                        actix_web::error::ErrorInternalServerError("Database error")
                    })?;

                if let Some(existing_contrib) = existing {
                    log::info!("🔄 [UPDATE] Found existing contribution for date {} repo {:?}: updating count {} -> {}",
                        contrib.date, contrib.repository_name, existing_contrib.count, contrib.count);

                    let mut active_model: contribution::ActiveModel = existing_contrib.into();
                    active_model.count = Set(contrib.count);
                    active_model.updated_at = Set(Utc::now());

                    active_model.update(db.as_ref()).await.map_err(|e| {
                        log::error!("❌ Failed to update contribution for date {}: {}", contrib.date, e);
                        actix_web::error::ErrorInternalServerError("Failed to update contribution")
                    })?;
                    total_updated += 1;
                    log::debug!("✅ Successfully updated contribution for date {}", contrib.date);
                } else {
                    log::info!("➕ [INSERT] New contribution for date {} repo {:?} with count {}",
                        contrib.date, contrib.repository_name, contrib.count);

                    let new_contrib = contribution::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        git_platform_account_id: Set(account_id),
                        contribution_date: Set(contrib.date),
                        count: Set(contrib.count),
                        repository_name: Set(contrib.repository_name.clone()),
                        is_private_repo: Set(contrib.is_private),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                    };

                    contribution::Entity::insert(new_contrib)
                        .exec(db.as_ref())
                        .await
                        .map_err(|e| {
                            log::error!("❌ Failed to insert contribution for date {}: {}", contrib.date, e);
                            actix_web::error::ErrorInternalServerError("Failed to insert contribution")
                        })?;
                    total_inserted += 1;
                    log::debug!("✅ Successfully inserted contribution for date {}", contrib.date);
                }
            }

            log::info!("💾 [Sync] Stored contributions: {} inserted, {} updated (total: {})",
                total_inserted, total_updated, total_inserted + total_updated);

            // Fetch and update profile data
            log::info!("👤 [Sync] Fetching Gitea profile data for {}", account.platform_username);
            match gitea_client.fetch_user_profile(&platform_config, &access_token).await {
                Ok(profile_data) => {
                    log::info!("✅ [Sync] Fetched Gitea profile data successfully");

                    let avatar_url = profile_data.get("avatar_url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let display_name = profile_data.get("full_name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let bio = profile_data.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let profile_url = Some(format!("{}/{}", instance_url, account.platform_username));

                    let location = profile_data.get("location")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let followers_count = profile_data.get("followers_count")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32);

                    let following_count = profile_data.get("following_count")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32);

                    // Update account with profile data
                    let account_for_update = git_platform_account::Entity::find_by_id(account_id)
                        .one(db.as_ref())
                        .await
                        .map_err(|e| {
                            log::error!("Database error: {}", e);
                            actix_web::error::ErrorInternalServerError("Database error")
                        })?
                        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

                    let mut account_update: git_platform_account::ActiveModel = account_for_update.into();

                    if let Some(url) = avatar_url {
                        account_update.avatar_url = Set(Some(url));
                    }
                    if let Some(name) = display_name {
                        account_update.display_name = Set(Some(name));
                    }
                    if let Some(b) = bio {
                        account_update.bio = Set(Some(b));
                    }
                    if let Some(url) = profile_url {
                        account_update.profile_url = Set(Some(url));
                    }
                    if let Some(loc) = location {
                        account_update.location = Set(Some(loc));
                    }
                    if let Some(count) = followers_count {
                        account_update.followers_count = Set(Some(count));
                    }
                    if let Some(count) = following_count {
                        account_update.following_count = Set(Some(count));
                    }

                    account_update.updated_at = Set(Utc::now());
                    account_update.update(db.as_ref()).await.map_err(|e| {
                        log::error!("Failed to update account: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to update account")
                    })?;

                    log::info!("💾 [Sync] Stored Gitea profile data successfully");
                }
                Err(e) => {
                    log::warn!("⚠️  [Sync] Failed to fetch Gitea profile data (continuing sync): {}", e);
                }
            }
        }
        _ => {
            return Err(actix_web::error::ErrorNotImplemented(
                "Syncing is only implemented for GitHub and Gitea currently"
            ));
        }
    }

    // Update last_synced_at timestamp
    let account_for_timestamp = git_platform_account::Entity::find_by_id(account_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

    let mut account: git_platform_account::ActiveModel = account_for_timestamp.into();
    account.last_synced_at = Set(Some(Utc::now()));
    account.updated_at = Set(Utc::now());

    account.update(db.as_ref()).await.map_err(|e| {
        log::error!("Failed to update sync timestamp: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update sync status")
    })?;

    log::info!("✅ [Sync] Sync completed successfully for account: {}", account_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Contributions synced successfully"
    })))
}
