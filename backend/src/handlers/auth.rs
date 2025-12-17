use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, ColumnTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{user, user_setting};
use crate::utils::auth::{create_jwt, hash_password, verify_password};
use crate::utils::config::Config;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    log::info!("📝 Registration attempt for username: {}", req.username);

    // Check if registration is allowed
    if !config.allow_registration {
        log::warn!("❌ Registration attempt rejected - registration is disabled");
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Registration is currently disabled".to_string(),
        });
    }

    // Check if username already exists
    let existing_user = user::Entity::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(db.get_ref())
        .await;

    match existing_user {
        Ok(Some(_)) => {
            log::warn!("❌ Registration failed - username '{}' already exists", req.username);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Username already exists".to_string(),
            });
        }
        Err(e) => {
            log::error!("❌ Database error during registration: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            });
        }
        _ => {}
    }

    // Hash password
    let password_hash = match hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("❌ Failed to hash password: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to hash password: {}", e),
            });
        }
    };

    // Create user
    log::info!("💾 Creating user '{}'...", req.username);
    let user_id = Uuid::new_v4();
    let new_user = user::ActiveModel {
        id: Set(user_id),
        username: Set(req.username.clone()),
        password_hash: Set(password_hash),
        email: Set(req.email.clone()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let user_result = new_user.insert(db.get_ref()).await;

    match user_result {
        Ok(user) => {
            log::info!("✅ User '{}' created successfully (ID: {})", user.username, user.id);

            // Create default user settings
            let user_settings = user_setting::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                show_private_contributions: Set(true),
                hide_private_repo_names: Set(false),
                heatmap_color_scheme: Set("green".to_string()),
                heatmap_size: Set("medium".to_string()),
                dark_mode_enabled: Set(false),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            if let Err(e) = user_settings.insert(db.get_ref()).await {
                log::error!("⚠️  Failed to create user settings: {}", e);
            } else {
                log::info!("✅ User settings created for '{}'", user.username);
            }

            // Generate JWT
            let token = match create_jwt(user.id, &config.jwt_secret, config.jwt_expiration_hours) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("❌ Failed to generate token: {}", e);
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate token: {}", e),
                    });
                }
            };

            log::info!("🎫 JWT token generated for user '{}'", user.username);

            HttpResponse::Created().json(AuthResponse {
                token,
                user_id: user.id.to_string(),
                username: user.username,
            })
        }
        Err(e) => {
            log::error!("❌ Failed to create user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            })
        }
    }
}

pub async fn login(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    log::info!("🔐 Login attempt for username: {}", req.username);

    // Find user by username
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(db.get_ref())
        .await;

    match user {
        Ok(Some(user)) => {
            log::info!("👤 User '{}' found, verifying password...", req.username);

            // Verify password
            match verify_password(&req.password, &user.password_hash) {
                Ok(true) => {
                    log::info!("✅ Password verified for user '{}'", req.username);

                    // Generate JWT
                    let token = match create_jwt(user.id, &config.jwt_secret, config.jwt_expiration_hours) {
                        Ok(t) => t,
                        Err(e) => {
                            log::error!("❌ Failed to generate token: {}", e);
                            return HttpResponse::InternalServerError().json(ErrorResponse {
                                error: format!("Failed to generate token: {}", e),
                            });
                        }
                    };

                    log::info!("🎫 JWT token generated for user '{}'", req.username);

                    HttpResponse::Ok().json(AuthResponse {
                        token,
                        user_id: user.id.to_string(),
                        username: user.username,
                    })
                }
                Ok(false) => {
                    log::warn!("❌ Invalid password for user '{}'", req.username);
                    HttpResponse::Unauthorized().json(ErrorResponse {
                        error: "Invalid credentials".to_string(),
                    })
                }
                Err(e) => {
                    log::error!("❌ Failed to verify password: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to verify password: {}", e),
                    })
                }
            }
        }
        Ok(None) => {
            log::warn!("❌ User '{}' not found", req.username);
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            })
        }
        Err(e) => {
            log::error!("❌ Database error during login: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
            })
        }
    }
}
