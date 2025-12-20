use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user_setting;

#[derive(Debug, Serialize)]
pub struct UserSettingsResponse {
    pub show_private_contributions: bool,
    pub hide_private_repo_names: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub show_private_contributions: Option<bool>,
    pub hide_private_repo_names: Option<bool>,
}

/// GET /api/settings
/// Get user settings
pub async fn get_settings(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let settings = user_setting::Entity::find()
        .filter(user_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    match settings {
        Some(settings) => Ok(HttpResponse::Ok().json(UserSettingsResponse {
            show_private_contributions: settings.show_private_contributions,
            hide_private_repo_names: settings.hide_private_repo_names,
            updated_at: settings.updated_at.to_rfc3339(),
        })),
        None => {
            // Return default settings if not found
            Ok(HttpResponse::Ok().json(UserSettingsResponse {
                show_private_contributions: true,
                hide_private_repo_names: false,
                updated_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
    }
}

/// PUT /api/settings
/// Update user settings
pub async fn update_settings(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<UpdateSettingsRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Check if settings exist
    let existing_settings = user_setting::Entity::find()
        .filter(user_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let updated_settings = if let Some(settings) = existing_settings {
        // Update existing settings
        let mut settings: user_setting::ActiveModel = settings.into();

        if let Some(show_private_contributions) = payload.show_private_contributions {
            settings.show_private_contributions = Set(show_private_contributions);
        }

        if let Some(hide_private_repo_names) = payload.hide_private_repo_names {
            settings.hide_private_repo_names = Set(hide_private_repo_names);
        }

        settings.updated_at = Set(chrono::Utc::now());

        settings.update(db.as_ref()).await.map_err(|e| {
            log::error!("Failed to update settings: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update settings")
        })?
    } else {
        // Create new settings
        let new_settings = user_setting::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            show_private_contributions: Set(payload
                .show_private_contributions
                .unwrap_or(true)),
            hide_private_repo_names: Set(payload.hide_private_repo_names.unwrap_or(false)),
            heatmap_color_scheme: Set("github".to_string()),
            heatmap_size: Set("medium".to_string()),
            dark_mode_enabled: Set(false),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        user_setting::Entity::insert(new_settings)
            .exec_with_returning(db.as_ref())
            .await
            .map_err(|e| {
                log::error!("Failed to create settings: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to create settings")
            })?
    };

    Ok(HttpResponse::Ok().json(UserSettingsResponse {
        show_private_contributions: updated_settings.show_private_contributions,
        hide_private_repo_names: updated_settings.hide_private_repo_names,
        updated_at: updated_settings.updated_at.to_rfc3339(),
    }))
}
