use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::Serialize;
use uuid::Uuid;

use crate::services::platform_sync::PlatformSyncService;

// ============ Response DTOs ============

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub platforms_synced: i32,
    pub contributions_added: i32,
    pub contributions_updated: i32,
    pub errors: Vec<String>,
}

// ============ Sync Handlers ============

/// POST /api/sync/trigger
/// Manually trigger sync for current user's all platforms
pub async fn trigger_sync(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    log::info!("Manual sync triggered for user: {}", user_id);

    let sync_service = PlatformSyncService::new(db.get_ref().clone());

    match sync_service.sync_user_data(user_id).await {
        Ok(result) => {
            let response = SyncResponse {
                success: result.errors.is_empty(),
                message: format!(
                    "Sync completed: {} platforms synced, {} contributions added, {} updated",
                    result.platforms_synced, result.contributions_added, result.contributions_updated
                ),
                platforms_synced: result.platforms_synced,
                contributions_added: result.contributions_added,
                contributions_updated: result.contributions_updated,
                errors: result.errors,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Sync failed for user {}: {}", user_id, e);
            let response = SyncResponse {
                success: false,
                message: format!("Sync failed: {}", e),
                platforms_synced: 0,
                contributions_added: 0,
                contributions_updated: 0,
                errors: vec![e.to_string()],
            };
            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}

/// GET /api/sync/status
/// Get sync status for current user
pub async fn get_sync_status(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Get generation settings to see last sync time
    let settings = crate::models::heatmap_generation_setting::Entity::find()
        .filter(crate::models::heatmap_generation_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Get platform accounts to see individual sync times
    let accounts = crate::models::git_platform_account::Entity::find()
        .filter(crate::models::git_platform_account::Column::UserId.eq(user_id))
        .filter(crate::models::git_platform_account::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    #[derive(Serialize)]
    struct SyncStatus {
        auto_sync_enabled: bool,
        update_interval_minutes: i32,
        last_sync: Option<String>,
        next_sync: Option<String>,
        platform_accounts: Vec<PlatformSyncStatus>,
    }

    #[derive(Serialize)]
    struct PlatformSyncStatus {
        platform: String,
        username: String,
        last_synced: Option<String>,
    }

    let status = SyncStatus {
        auto_sync_enabled: settings.as_ref().map(|s| s.auto_generation_enabled).unwrap_or(false),
        update_interval_minutes: settings.as_ref().map(|s| s.update_interval_minutes).unwrap_or(60),
        last_sync: settings.as_ref().and_then(|s| s.last_scheduled_generation_at.map(|t| t.to_rfc3339())),
        next_sync: settings.as_ref().and_then(|s| s.next_scheduled_generation_at.map(|t| t.to_rfc3339())),
        platform_accounts: accounts
            .into_iter()
            .map(|acc| PlatformSyncStatus {
                platform: format!("{:?}", acc.platform_type),
                username: acc.platform_username,
                last_synced: acc.last_synced_at.map(|t| t.to_rfc3339()),
            })
            .collect(),
    };

    Ok(HttpResponse::Ok().json(status))
}
