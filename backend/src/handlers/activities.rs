use actix_web::{web, HttpResponse, Responder};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{activity, git_platform_account, user, user_setting};

#[derive(Debug, Deserialize)]
pub struct ActivitiesQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: String,
    pub activity_type: String,
    pub date: String,
    pub metadata: serde_json::Value,
    pub repository_name: Option<String>,
    pub repository_url: Option<String>,
    pub is_private: bool,
    pub count: i32,
    pub primary_language: Option<String>,
    pub organization_name: Option<String>,
    pub organization_avatar_url: Option<String>,
    // Platform information for generating correct URLs
    pub platform: String,
    pub platform_username: String,
    pub platform_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivitiesResponse {
    pub activities: Vec<ActivityResponse>,
    pub total: i32,
    pub has_more: bool,
}

/// GET /api/activities
/// Get user's activity timeline
pub async fn get_activities(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<ActivitiesQuery>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    // Get user settings for privacy filtering
    let settings = user_setting::Entity::find()
        .filter(user_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let show_private_contributions = settings
        .as_ref()
        .map(|s| s.show_private_contributions)
        .unwrap_or(true);

    let hide_private_repo_names = settings
        .as_ref()
        .map(|s| s.hide_private_repo_names)
        .unwrap_or(false);

    // Get all active platform accounts for this user
    let mut accounts_query = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true));

    // Filter by platform type if specified
    if let Some(platform_filter) = &query.platform {
        let platform_type = match platform_filter.to_lowercase().as_str() {
            "github" => git_platform_account::GitPlatform::GitHub,
            "gitea" => git_platform_account::GitPlatform::Gitea,
            "gitlab" => git_platform_account::GitPlatform::GitLab,
            _ => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid platform filter: {}", platform_filter)
                })));
            }
        };
        accounts_query =
            accounts_query.filter(git_platform_account::Column::PlatformType.eq(platform_type));
    }

    let accounts = accounts_query.all(db.as_ref()).await.map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if accounts.is_empty() {
        return Ok(HttpResponse::Ok().json(ActivitiesResponse {
            activities: vec![],
            total: 0,
            has_more: false,
        }));
    }

    let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

    // Create a HashMap for quick platform account lookups
    let accounts_map: std::collections::HashMap<Uuid, &git_platform_account::Model> =
        accounts.iter().map(|a| (a.id, a)).collect();

    // Build query for activities
    let mut activity_query = activity::Entity::find()
        .filter(activity::Column::GitPlatformAccountId.is_in(account_ids))
        .order_by_desc(activity::Column::ActivityDate);

    // Apply privacy filter
    if !show_private_contributions {
        activity_query = activity_query.filter(activity::Column::IsPrivateRepo.eq(false));
    }

    // Apply date range if provided
    if let Some(from_str) = &query.from {
        if let Ok(from_date) = chrono::NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.gte(from_date));
        }
    }

    if let Some(to_str) = &query.to {
        if let Ok(to_date) = chrono::NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.lte(to_date));
        }
    }

    // Count total activities before pagination
    let total = activity_query
        .clone()
        .count(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Apply pagination
    let limit = query.limit.unwrap_or(50).min(100) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;

    activity_query = activity_query.limit(limit).offset(offset);

    let activities = activity_query.all(db.as_ref()).await.map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let has_more = (offset + limit) < total;

    let activity_responses: Vec<ActivityResponse> = activities
        .into_iter()
        .filter_map(|a| {
            // Get platform account info for this activity
            let account = accounts_map.get(&a.git_platform_account_id)?;

            // Hide repository name if it's private and user wants to hide private repo names
            let repository_name = if a.is_private_repo && hide_private_repo_names {
                None
            } else {
                a.repository_name
            };

            // Sanitize metadata to hide repository names in the repositories array for Commit activities
            let metadata = if a.is_private_repo && hide_private_repo_names {
                // Clone and modify metadata to hide repository names
                let mut sanitized_metadata = a.metadata.clone();
                if let Some(repos) = sanitized_metadata.get_mut("repositories") {
                    if let Some(repos_array) = repos.as_array_mut() {
                        for repo in repos_array.iter_mut() {
                            if let Some(repo_obj) = repo.as_object_mut() {
                                repo_obj.insert(
                                    "name".to_string(),
                                    serde_json::json!("Private Repository"),
                                );
                            }
                        }
                    }
                }
                sanitized_metadata
            } else {
                a.metadata
            };

            Some(ActivityResponse {
                id: a.id.to_string(),
                activity_type: format!("{:?}", a.activity_type),
                date: a.activity_date.format("%Y-%m-%d").to_string(),
                metadata,
                repository_name,
                repository_url: a.repository_url,
                is_private: a.is_private_repo,
                count: a.count,
                primary_language: a.primary_language,
                organization_name: a.organization_name,
                organization_avatar_url: a.organization_avatar_url,
                platform: format!("{:?}", account.platform_type).to_lowercase(),
                platform_username: account.platform_username.clone(),
                platform_url: account.platform_url.clone(),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(ActivitiesResponse {
        activities: activity_responses,
        total: total as i32,
        has_more,
    }))
}

/// GET /api/users/:username/activities
/// Public endpoint to get user's activity timeline by username
pub async fn get_user_activities(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    query: web::Query<ActivitiesQuery>,
) -> Result<impl Responder, actix_web::Error> {
    let username = path.into_inner();

    // Find user by username (case-insensitive)
    let user_model = user::Entity::find()
        .filter(
            Expr::expr(Func::lower(Expr::col(user::Column::Username))).eq(username.to_lowercase()),
        )
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let user_model = match user_model {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })));
        }
    };

    let user_id = user_model.id;

    // Get user settings for privacy filtering
    let settings = user_setting::Entity::find()
        .filter(user_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let show_private_contributions = settings
        .as_ref()
        .map(|s| s.show_private_contributions)
        .unwrap_or(true);

    let hide_private_repo_names = settings
        .as_ref()
        .map(|s| s.hide_private_repo_names)
        .unwrap_or(false);

    // Get all active platform accounts for this user
    let mut accounts_query = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true));

    // Filter by platform type if specified
    if let Some(platform_filter) = &query.platform {
        let platform_type = match platform_filter.to_lowercase().as_str() {
            "github" => git_platform_account::GitPlatform::GitHub,
            "gitea" => git_platform_account::GitPlatform::Gitea,
            "gitlab" => git_platform_account::GitPlatform::GitLab,
            _ => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid platform filter: {}", platform_filter)
                })));
            }
        };
        accounts_query =
            accounts_query.filter(git_platform_account::Column::PlatformType.eq(platform_type));
    }

    let accounts = accounts_query.all(db.as_ref()).await.map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    if accounts.is_empty() {
        return Ok(HttpResponse::Ok().json(ActivitiesResponse {
            activities: vec![],
            total: 0,
            has_more: false,
        }));
    }

    let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

    // Create a HashMap for quick platform account lookups
    let accounts_map: std::collections::HashMap<Uuid, &git_platform_account::Model> =
        accounts.iter().map(|a| (a.id, a)).collect();

    // Build query for activities
    let mut activity_query = activity::Entity::find()
        .filter(activity::Column::GitPlatformAccountId.is_in(account_ids))
        .order_by_desc(activity::Column::ActivityDate);

    // Apply privacy filter
    if !show_private_contributions {
        activity_query = activity_query.filter(activity::Column::IsPrivateRepo.eq(false));
    }

    // Apply date range if provided
    if let Some(from_str) = &query.from {
        if let Ok(from_date) = chrono::NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.gte(from_date));
        }
    }

    if let Some(to_str) = &query.to {
        if let Ok(to_date) = chrono::NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.lte(to_date));
        }
    }

    // Count total activities before pagination
    let total = activity_query
        .clone()
        .count(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Apply pagination
    let limit = query.limit.unwrap_or(50).min(100) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;

    activity_query = activity_query.limit(limit).offset(offset);

    let activities = activity_query.all(db.as_ref()).await.map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let has_more = (offset + limit) < total;

    let activity_responses: Vec<ActivityResponse> = activities
        .into_iter()
        .filter_map(|a| {
            // Get platform account info for this activity
            let account = accounts_map.get(&a.git_platform_account_id)?;

            // Hide repository name if it's private and user wants to hide private repo names
            let repository_name = if a.is_private_repo && hide_private_repo_names {
                None
            } else {
                a.repository_name
            };

            // Sanitize metadata to hide repository names in the repositories array for Commit activities
            let metadata = if a.is_private_repo && hide_private_repo_names {
                // Clone and modify metadata to hide repository names
                let mut sanitized_metadata = a.metadata.clone();
                if let Some(repos) = sanitized_metadata.get_mut("repositories") {
                    if let Some(repos_array) = repos.as_array_mut() {
                        for repo in repos_array.iter_mut() {
                            if let Some(repo_obj) = repo.as_object_mut() {
                                repo_obj.insert(
                                    "name".to_string(),
                                    serde_json::json!("Private Repository"),
                                );
                            }
                        }
                    }
                }
                sanitized_metadata
            } else {
                a.metadata
            };

            Some(ActivityResponse {
                id: a.id.to_string(),
                activity_type: format!("{:?}", a.activity_type),
                date: a.activity_date.format("%Y-%m-%d").to_string(),
                metadata,
                repository_name,
                repository_url: a.repository_url,
                is_private: a.is_private_repo,
                count: a.count,
                primary_language: a.primary_language,
                organization_name: a.organization_name,
                organization_avatar_url: a.organization_avatar_url,
                platform: format!("{:?}", account.platform_type).to_lowercase(),
                platform_username: account.platform_username.clone(),
                platform_url: account.platform_url.clone(),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(ActivitiesResponse {
        activities: activity_responses,
        total: total as i32,
        has_more,
    }))
}
