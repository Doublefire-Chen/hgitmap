use chrono::{Duration, Utc};
use sea_orm::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

use crate::models::{git_platform_account, heatmap_generation_setting, platform_sync_job};
use crate::utils::config::Config;

pub struct SyncScheduler {
    db: DatabaseConnection,
    #[allow(dead_code)]
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
                log::info!("📅 [SyncScheduler] Scheduling sync for user: {}", user_id);

                // Get all active platform accounts for this user
                let accounts = git_platform_account::Entity::find()
                    .filter(git_platform_account::Column::UserId.eq(user_id))
                    .filter(git_platform_account::Column::IsActive.eq(true))
                    .all(&self.db)
                    .await?;

                if accounts.is_empty() {
                    log::debug!(
                        "📅 [SyncScheduler] No active platform accounts for user {}",
                        user_id
                    );
                    continue;
                }

                let mut jobs_created = 0;

                // Create a sync job for each active platform account
                for account in accounts {
                    // Check if there's already a pending or processing job for this account
                    let existing_job = platform_sync_job::Entity::find()
                        .filter(platform_sync_job::Column::PlatformAccountId.eq(account.id))
                        .filter(
                            platform_sync_job::Column::Status
                                .eq(platform_sync_job::SyncJobStatus::Pending)
                                .or(platform_sync_job::Column::Status
                                    .eq(platform_sync_job::SyncJobStatus::Processing)),
                        )
                        .one(&self.db)
                        .await?;

                    if existing_job.is_some() {
                        log::debug!(
                            "📅 [SyncScheduler] Skipping {} - job already pending/processing",
                            account.platform_username
                        );
                        continue;
                    }

                    // Create a new sync job
                    let job = platform_sync_job::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        user_id: Set(user_id),
                        platform_account_id: Set(account.id),
                        status: Set(platform_sync_job::SyncJobStatus::Pending),
                        sync_all_years: Set(false), // Scheduled syncs only sync current year
                        specific_year: Set(None),
                        sync_contributions: Set(account.sync_contributions),
                        sync_activities: Set(account.sync_contributions), // Activities sync with contributions
                        sync_profile: Set(account.sync_profile),
                        scheduled_at: Set(now),
                        started_at: Set(None),
                        completed_at: Set(None),
                        error_message: Set(None),
                        retry_count: Set(0),
                        max_retries: Set(3),
                        contributions_synced: Set(None),
                        activities_synced: Set(None),
                        years_completed: Set(None),
                        total_years: Set(None),
                        is_manual: Set(false), // This is an automatic scheduled sync
                        priority: Set(0),      // Lower priority than manual syncs
                        created_at: Set(now),
                    };

                    platform_sync_job::Entity::insert(job)
                        .exec(&self.db)
                        .await?;

                    jobs_created += 1;
                    log::info!(
                        "📅 [SyncScheduler] Created sync job for {} ({:?})",
                        account.platform_username,
                        account.platform_type
                    );
                }

                if jobs_created > 0 {
                    // Update last_scheduled_generation_at now that we've queued the jobs
                    if let Err(e) = update_sync_timestamps(&self.db, user_id).await {
                        log::error!("Failed to update sync timestamps: {}", e);
                    }

                    log::info!(
                        "📅 [SyncScheduler] Created {} sync jobs for user {}",
                        jobs_created,
                        user_id
                    );
                }
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
