use chrono::{Duration, Utc};
use sea_orm::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};

use crate::models::heatmap_generation_setting;
use crate::services::platform_sync::PlatformSyncService;
use crate::utils::config::Config;

pub struct SyncScheduler {
    db: DatabaseConnection,
    config: Config,
    check_interval_seconds: u64,
}

impl SyncScheduler {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        Self {
            db,
            config,
            check_interval_seconds: 60, // Check every minute
        }
    }

    /// Start the background scheduler
    pub async fn start(self: Arc<Self>) {
        log::info!("Starting sync scheduler (checking every {} seconds)", self.check_interval_seconds);

        loop {
            if let Err(e) = self.check_and_sync_users().await {
                log::error!("Error in sync scheduler: {}", e);
            }

            sleep(TokioDuration::from_secs(self.check_interval_seconds)).await;
        }
    }

    /// Check all users and sync those that are due
    async fn check_and_sync_users(&self) -> Result<(), DbErr> {
        let now = Utc::now();

        // Find all users with auto_generation_enabled
        let settings = heatmap_generation_setting::Entity::find()
            .filter(heatmap_generation_setting::Column::AutoGenerationEnabled.eq(true))
            .all(&self.db)
            .await?;

        for setting in settings {
            // Check if it's time to sync this user
            let should_sync = match setting.last_scheduled_generation_at {
                Some(last_sync) => {
                    let next_sync = last_sync + Duration::minutes(setting.update_interval_minutes as i64);
                    now >= next_sync
                }
                None => true, // Never synced before, sync now
            };

            if should_sync {
                let user_id = setting.user_id;
                log::info!("Scheduling sync for user: {}", user_id);

                // Spawn a new task for this sync to avoid blocking
                let db_clone = self.db.clone();
                let config_clone = self.config.clone();
                tokio::spawn(async move {
                    let sync_service = PlatformSyncService::new(db_clone.clone(), config_clone);

                    match sync_service.sync_user_data(user_id).await {
                        Ok(result) => {
                            log::info!(
                                "Sync completed for user {}: {} platforms, {} added, {} updated",
                                user_id,
                                result.platforms_synced,
                                result.contributions_added,
                                result.contributions_updated
                            );

                            // Update last_scheduled_generation_at and next_scheduled_generation_at
                            if let Err(e) = update_sync_timestamps(&db_clone, user_id).await {
                                log::error!("Failed to update sync timestamps: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Sync failed for user {}: {}", user_id, e);
                        }
                    }
                });
            }
        }

        Ok(())
    }
}

/// Update the sync timestamps in the database
async fn update_sync_timestamps(db: &DatabaseConnection, user_id: uuid::Uuid) -> Result<(), DbErr> {
    let setting = heatmap_generation_setting::Entity::find()
        .filter(heatmap_generation_setting::Column::UserId.eq(user_id))
        .one(db)
        .await?;

    if let Some(setting) = setting {
        let now = Utc::now();
        let next_sync = now + Duration::minutes(setting.update_interval_minutes as i64);

        let mut active_setting: heatmap_generation_setting::ActiveModel = setting.into();
        active_setting.last_scheduled_generation_at = Set(Some(now));
        active_setting.next_scheduled_generation_at = Set(Some(next_sync));
        active_setting.updated_at = Set(now);
        active_setting.update(db).await?;
    }

    Ok(())
}
