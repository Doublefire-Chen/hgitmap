use actix_web::{web, HttpResponse, Responder};
use sea_orm::{*, sea_query::Expr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{oauth_application, user, git_platform_account};
use crate::utils::{config::Config, encryption};

#[derive(Debug, Deserialize)]
pub struct CreateOAuthAppRequest {
    pub platform: String,
    pub instance_url: Option<String>,
    pub instance_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOAuthAppRequest {
    pub instance_name: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub is_enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct OAuthAppResponse {
    pub id: String,
    pub platform: String,
    pub instance_url: String,
    pub instance_name: String,
    pub client_id: String,
    pub client_secret_preview: String, // Only show first/last few chars
    pub is_enabled: bool,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Middleware-like function to check if user is admin
async fn require_admin(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(), actix_web::Error> {
    let user = user::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    if !user.is_admin {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }

    Ok(())
}

fn mask_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        return "••••••••".to_string();
    }
    let start = &secret[..4];
    let end = &secret[secret.len() - 4..];
    format!("{}••••{}", start, end)
}

/// GET /api/admin/oauth-apps
/// List all OAuth applications (admin only)
pub async fn list_oauth_apps(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    require_admin(db.as_ref(), user_id).await?;

    let apps = oauth_application::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let response: Vec<OAuthAppResponse> = apps
        .into_iter()
        .map(|app| {
            let platform_str = match app.platform {
                git_platform_account::GitPlatform::GitHub => "github",
                git_platform_account::GitPlatform::GitLab => "gitlab",
                git_platform_account::GitPlatform::Gitea => "gitea",
            };

            OAuthAppResponse {
                id: app.id.to_string(),
                platform: platform_str.to_string(),
                instance_url: app.instance_url,
                instance_name: app.instance_name,
                client_id: app.client_id,
                client_secret_preview: mask_secret(&app.client_secret),
                is_enabled: app.is_enabled,
                is_default: app.is_default,
                created_at: app.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /admin/oauth-apps
/// Create a new OAuth application (admin only)
pub async fn create_oauth_app(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<CreateOAuthAppRequest>,
) -> Result<impl Responder, actix_web::Error> {
    log::info!("🔧 [Admin] Create OAuth app request received");
    log::debug!("Payload: {:?}", payload);
    log::info!("User ID: {}", user_claims.sub);

    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        log::error!("❌ Invalid user ID: {}", e);
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    log::info!("Checking admin privileges");
    require_admin(db.as_ref(), user_id).await?;
    log::info!("✅ Admin check passed");

    // Validate platform
    log::info!("Validating platform: {}", payload.platform);
    let platform = match payload.platform.to_lowercase().as_str() {
        "github" => git_platform_account::GitPlatform::GitHub,
        "gitlab" => git_platform_account::GitPlatform::GitLab,
        "gitea" => git_platform_account::GitPlatform::Gitea,
        _ => {
            log::error!("❌ Invalid platform: {}", payload.platform);
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid platform. Must be github, gitlab, or gitea".to_string(),
            }))
        }
    };

    // Normalize instance URL
    let instance_url = payload.instance_url.clone().unwrap_or_default();
    log::info!("Instance URL: '{}'", instance_url);

    // Check if OAuth app already exists for this platform + instance
    log::info!("Checking for existing OAuth app");
    let existing = oauth_application::Entity::find()
        .filter(oauth_application::Column::Platform.eq(platform.clone()))
        .filter(oauth_application::Column::InstanceUrl.eq(&instance_url))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if existing.is_some() {
        log::warn!("⚠️  OAuth app already exists for this platform and instance");
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "OAuth application already exists for this platform and instance".to_string(),
        }));
    }

    // Encrypt the client secret
    log::info!("🔒 Encrypting client secret");
    let encrypted_secret = encryption::encrypt(&payload.client_secret, &config.encryption_key)
        .map_err(|e| {
            log::error!("Failed to encrypt secret: {}", e);
            actix_web::error::ErrorInternalServerError("Encryption failed")
        })?;

    // If this is set as default, unset other defaults for this platform
    if payload.is_default.unwrap_or(false) {
        log::info!("Unsetting other default apps for this platform");
        oauth_application::Entity::update_many()
            .col_expr(
                oauth_application::Column::IsDefault,
                Expr::value(false),
            )
            .filter(oauth_application::Column::Platform.eq(platform.clone()))
            .exec(db.as_ref())
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
    }

    log::info!("💾 Creating new OAuth app in database");
    let new_app = oauth_application::ActiveModel {
        id: Set(Uuid::new_v4()),
        platform: Set(platform),
        instance_url: Set(instance_url),
        instance_name: Set(payload.instance_name.clone()),
        client_id: Set(payload.client_id.clone()),
        client_secret: Set(encrypted_secret),
        is_enabled: Set(true),
        is_default: Set(payload.is_default.unwrap_or(false)),
        created_by: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let app = oauth_application::Entity::insert(new_app)
        .exec_with_returning(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to create OAuth app: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create OAuth app")
        })?;

    log::info!("✅ OAuth app created successfully: {}", app.id);

    let platform_str = match app.platform {
        git_platform_account::GitPlatform::GitHub => "github",
        git_platform_account::GitPlatform::GitLab => "gitlab",
        git_platform_account::GitPlatform::Gitea => "gitea",
    };

    Ok(HttpResponse::Ok().json(OAuthAppResponse {
        id: app.id.to_string(),
        platform: platform_str.to_string(),
        instance_url: app.instance_url,
        instance_name: app.instance_name,
        client_id: app.client_id,
        client_secret_preview: mask_secret(&payload.client_secret),
        is_enabled: app.is_enabled,
        is_default: app.is_default,
        created_at: app.created_at.to_rfc3339(),
    }))
}

/// PUT /api/admin/oauth-apps/:id
/// Update an OAuth application (admin only)
pub async fn update_oauth_app(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    payload: web::Json<UpdateOAuthAppRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    require_admin(db.as_ref(), user_id).await?;

    let app_id = Uuid::parse_str(&path.into_inner()).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid app ID: {}", e))
    })?;

    let app = oauth_application::Entity::find_by_id(app_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("OAuth app not found"))?;

    let mut app: oauth_application::ActiveModel = app.into();

    if let Some(instance_name) = &payload.instance_name {
        app.instance_name = Set(instance_name.clone());
    }

    if let Some(client_id) = &payload.client_id {
        app.client_id = Set(client_id.clone());
    }

    if let Some(client_secret) = &payload.client_secret {
        let encrypted_secret = encryption::encrypt(client_secret, &config.encryption_key)
            .map_err(|e| {
                log::error!("Failed to encrypt secret: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption failed")
            })?;
        app.client_secret = Set(encrypted_secret);
    }

    if let Some(is_enabled) = payload.is_enabled {
        app.is_enabled = Set(is_enabled);
    }

    if let Some(is_default) = payload.is_default {
        if is_default {
            // Unset other defaults for this platform
            let platform = app.platform.clone().unwrap();
            oauth_application::Entity::update_many()
                .col_expr(
                    oauth_application::Column::IsDefault,
                    Expr::value(false),
                )
                .filter(oauth_application::Column::Platform.eq(platform))
                .exec(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?;
        }
        app.is_default = Set(is_default);
    }

    app.updated_at = Set(chrono::Utc::now());

    let updated_app = app.update(db.as_ref()).await.map_err(|e| {
        log::error!("Failed to update OAuth app: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update OAuth app")
    })?;

    let platform_str = match updated_app.platform {
        git_platform_account::GitPlatform::GitHub => "github",
        git_platform_account::GitPlatform::GitLab => "gitlab",
        git_platform_account::GitPlatform::Gitea => "gitea",
    };

    Ok(HttpResponse::Ok().json(OAuthAppResponse {
        id: updated_app.id.to_string(),
        platform: platform_str.to_string(),
        instance_url: updated_app.instance_url,
        instance_name: updated_app.instance_name,
        client_id: updated_app.client_id,
        client_secret_preview: mask_secret(&updated_app.client_secret),
        is_enabled: updated_app.is_enabled,
        is_default: updated_app.is_default,
        created_at: updated_app.created_at.to_rfc3339(),
    }))
}

/// DELETE /api/admin/oauth-apps/:id
/// Delete an OAuth application (admin only)
pub async fn delete_oauth_app(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    require_admin(db.as_ref(), user_id).await?;

    let app_id = Uuid::parse_str(&path.into_inner()).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid app ID: {}", e))
    })?;

    oauth_application::Entity::delete_by_id(app_id)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to delete OAuth app: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to delete OAuth app")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OAuth application deleted successfully"
    })))
}
