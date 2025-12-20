use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sea_orm::*;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::models::{git_platform_account, oauth_application, oauth_state};
use crate::services::git_platforms::{github::GitHubClient, GitPlatform, PlatformConfig};
use crate::utils::{config::Config, encryption};

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizeQuery {
    pub instance_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
struct OAuthErrorResponse {
    error: String,
}

/// Helper function to get OAuth credentials for a platform from database
async fn get_oauth_credentials(
    db: &DatabaseConnection,
    config: &Config,
    platform: git_platform_account::GitPlatform,
    instance_url: &str,
) -> Result<(String, String), actix_web::Error> {
    // Get from database
    let oauth_app = oauth_application::Entity::find()
        .filter(oauth_application::Column::Platform.eq(platform.clone()))
        .filter(oauth_application::Column::InstanceUrl.eq(instance_url))
        .filter(oauth_application::Column::IsEnabled.eq(true))
        .order_by_desc(oauth_application::Column::IsDefault) // Prefer default
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error fetching OAuth app: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(app) = oauth_app {
        // Decrypt the client secret
        let client_secret = encryption::decrypt(&app.client_secret, &config.encryption_key)
            .map_err(|e| {
                log::error!("Failed to decrypt OAuth client secret: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to decrypt credentials")
            })?;

        log::info!("Using OAuth app '{}' from database", app.instance_name);
        Ok((app.client_id, client_secret))
    } else {
        // No OAuth app configured in database
        let platform_name = match platform {
            git_platform_account::GitPlatform::GitHub => "GitHub",
            git_platform_account::GitPlatform::GitLab => "GitLab",
            git_platform_account::GitPlatform::Gitea => "Gitea",
        };

        log::error!(
            "No {} OAuth application configured. Please configure via admin panel at /admin/oauth-apps",
            platform_name
        );

        Err(actix_web::error::ErrorServiceUnavailable(format!(
            "{} OAuth is not configured. Please contact the administrator to set up OAuth at /admin/oauth-apps",
            platform_name
        )))
    }
}

/// GET /oauth/github/authorize
/// Returns GitHub OAuth authorization URL with state token
pub async fn github_authorize(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    query: web::Query<OAuthAuthorizeQuery>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    log::info!("🔐 GitHub OAuth authorize request received");
    log::debug!("Query params: {:?}", query);

    // Get user ID from JWT
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        log::error!("❌ Invalid user ID: {}", e);
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    log::info!("User ID from JWT: {}", user_id);

    // GitHub only supports github.com, not self-hosted
    if query.instance_url.is_some() {
        log::warn!("GitHub does not support custom instance URLs");
        return Ok(HttpResponse::BadRequest().json(OAuthErrorResponse {
            error: "GitHub does not support custom instance URLs".to_string(),
        }));
    }

    // Get OAuth credentials from database
    log::info!("Fetching OAuth credentials from database");
    let (client_id, _) = get_oauth_credentials(
        db.as_ref(),
        &config,
        git_platform_account::GitPlatform::GitHub,
        "", // Empty string for official GitHub
    ).await?;

    // Generate state token for CSRF protection
    let state_token = format!("{}", Uuid::new_v4());
    log::info!("🎲 Generated state token: {}", state_token);

    // Store state in database with 10 minute expiration
    let oauth_state = oauth_state::ActiveModel {
        id: Set(Uuid::new_v4()),
        state_token: Set(state_token.clone()),
        user_id: Set(user_id),
        platform: Set(git_platform_account::GitPlatform::GitHub),
        created_at: Set(Utc::now()),
        expires_at: Set(Utc::now() + Duration::minutes(10)),
    };

    oauth_state::Entity::insert(oauth_state)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Failed to store OAuth state: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create OAuth state")
        })?;

    log::info!("💾 OAuth state stored in database");

    let redirect_uri = format!("{}/oauth/github/callback", config.base_url);
    log::info!("📍 Redirect URI: {}", redirect_uri);

    // Scopes: read:user for profile, repo for private contributions
    // Note: 'repo' scope includes write access but we only use it to read contribution data
    let scopes = "read:user,repo";

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scopes),
        urlencoding::encode(&state_token)
    );

    log::info!("🔗 Generated GitHub authorization URL");

    // Return the URL as JSON instead of redirecting
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "authorization_url": auth_url
    })))
}

/// GET /oauth/github/callback
/// Handles OAuth callback from GitHub
pub async fn github_callback(
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    query: web::Query<GitHubCallbackQuery>,
) -> Result<impl Responder, actix_web::Error> {
    log::info!("🔄 GitHub OAuth callback received");
    log::debug!("Code: {}...", &query.code.chars().take(10).collect::<String>());
    log::debug!("State: {:?}", query.state);

    let code = &query.code;

    // Validate state token
    let state_token = query.state.as_ref().ok_or_else(|| {
        log::error!("❌ Missing state parameter");
        actix_web::error::ErrorBadRequest("Missing state parameter")
    })?;

    log::info!("🔍 Validating state token");

    // Find and validate state in database
    let oauth_state = oauth_state::Entity::find()
        .filter(oauth_state::Column::StateToken.eq(state_token))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| {
            log::error!("❌ Invalid state token");
            actix_web::error::ErrorBadRequest("Invalid state token")
        })?;

    // Check if state has expired
    if oauth_state.expires_at < Utc::now() {
        log::error!("❌ State token has expired");
        // Clean up expired state
        oauth_state::Entity::delete_by_id(oauth_state.id)
            .exec(db.as_ref())
            .await
            .ok();
        return Err(actix_web::error::ErrorBadRequest("State token has expired"));
    }

    let user_id = oauth_state.user_id;
    log::info!("✅ State validated for user: {}", user_id);

    // Delete the used state token
    oauth_state::Entity::delete_by_id(oauth_state.id)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("⚠️  Failed to delete state token: {}", e);
            e
        })
        .ok();

    // Get OAuth credentials from database
    log::info!("Fetching OAuth credentials for token exchange");
    let (client_id, client_secret) = get_oauth_credentials(
        db.as_ref(),
        &config,
        git_platform_account::GitPlatform::GitHub,
        "", // Empty string for official GitHub
    ).await?;

    log::debug!("Client ID: {}...", &client_id.chars().take(10).collect::<String>());

    // Exchange code for access token
    log::info!("🔑 Exchanging authorization code for access token");
    let token_result = exchange_github_code(code, &client_id, &client_secret).await.map_err(|e| {
        log::error!("❌ Failed to exchange GitHub code: {}", e);
        actix_web::error::ErrorInternalServerError(format!("OAuth exchange failed: {}", e))
    })?;

    log::info!("✅ Successfully exchanged code for access token");

    // Validate token and get user info
    log::info!("👤 Validating token and fetching user info");
    let github_client = GitHubClient::new();
    let platform_config = PlatformConfig::github();

    let user_info = github_client
        .validate_token(&platform_config, &token_result.access_token)
        .await
        .map_err(|e| {
            log::error!("❌ Failed to validate GitHub token: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Token validation failed: {}", e))
        })?;

    log::info!("✅ GitHub user validated: {}", user_info.username);

    // Encrypt the access token
    log::info!("🔒 Encrypting access token");
    let encrypted_token = encryption::encrypt(&token_result.access_token, &config.encryption_key)
        .map_err(|e| {
            log::error!("❌ Failed to encrypt token: {}", e);
            actix_web::error::ErrorInternalServerError("Encryption failed")
        })?;

    // Check if this platform account already exists
    log::info!("🔍 Checking for existing platform account");
    let existing_account = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::PlatformType.eq(git_platform_account::GitPlatform::GitHub))
        .filter(git_platform_account::Column::PlatformUsername.eq(&user_info.username))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(account) = existing_account {
        log::info!("📝 Updating existing account");
        // Update existing account
        let mut account: git_platform_account::ActiveModel = account.into();
        account.access_token = Set(Some(encrypted_token));
        account.refresh_token = Set(None);
        account.is_active = Set(true);
        account.updated_at = Set(chrono::Utc::now());

        account.update(db.as_ref()).await.map_err(|e| {
            log::error!("❌ Failed to update account: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update account")
        })?;
        log::info!("✅ Account updated successfully");
    } else {
        log::info!("✨ Creating new platform account");
        // Create new account
        let new_account = git_platform_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            platform_type: Set(git_platform_account::GitPlatform::GitHub),
            platform_username: Set(user_info.username.clone()),
            access_token: Set(Some(encrypted_token)),
            refresh_token: Set(None),
            platform_url: Set(None),
            is_active: Set(true),
            last_synced_at: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        git_platform_account::Entity::insert(new_account)
            .exec(db.as_ref())
            .await
            .map_err(|e| {
                log::error!("❌ Failed to create account: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create account")
            })?;
        log::info!("✅ Account created successfully");
    }

    // Redirect to frontend with success
    let redirect_url = format!("{}/?oauth=success", config.base_url.replace(":3000", ":5173"));
    log::info!("🎉 OAuth flow complete! Redirecting to: {}", redirect_url);

    Ok(HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish())
}

/// Exchange GitHub authorization code for access token
async fn exchange_github_code(code: &str, client_id: &str, client_secret: &str) -> Result<GitHubTokenResponse> {
    log::info!("📤 Sending token exchange request to GitHub");

    let client = crate::utils::http_client::create_http_client();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
    ];

    log::debug!("Request params: client_id={}, code={}...",
        &client_id.chars().take(10).collect::<String>(),
        &code.chars().take(10).collect::<String>()
    );

    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    let status = response.status();
    log::info!("📥 GitHub response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await?;
        log::error!("❌ GitHub token exchange failed: {}", error_text);
        return Err(anyhow!(
            "GitHub token exchange failed with status {}: {}",
            status,
            error_text
        ));
    }

    let response_text = response.text().await?;
    log::debug!("Response body: {}", response_text);

    let token_response: GitHubTokenResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            log::error!("❌ Failed to parse GitHub response: {}", e);
            anyhow!("Failed to parse GitHub response: {}", e)
        })?;

    if let Some(error) = &token_response.error {
        log::error!("❌ GitHub OAuth error: {} - {:?}", error, token_response.error_description);
        return Err(anyhow!("GitHub OAuth error: {}", error));
    }

    log::info!("✅ Successfully received access token from GitHub");

    Ok(token_response)
}

#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    token_type: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}
