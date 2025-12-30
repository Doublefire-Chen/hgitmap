use chrono::{Datelike, Utc};
use sea_orm::*;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::models::{
    activity, contribution, git_platform_account,
    platform_sync_job::{self, SyncJobStatus},
};
use crate::services::activity_aggregation::ActivityAggregationService;
use crate::services::git_platforms::{
    GitHubClient, GitLabClient, GitPlatform, GiteaClient, PlatformConfig,
};
use crate::utils::encryption::decrypt;

pub struct SyncJobProcessor {
    db: DatabaseConnection,
    encryption_key: String,
    check_interval: Duration,
}

impl SyncJobProcessor {
    pub fn new(db: DatabaseConnection, encryption_key: String, check_interval_secs: u64) -> Self {
        Self {
            db,
            encryption_key,
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    /// Start the job processor loop
    pub async fn start(self: Arc<Self>) {
        log::info!("Starting platform sync job processor");

        // Reset any jobs that were stuck in "processing" state from previous shutdown
        if let Err(e) = self.reset_stale_jobs().await {
            log::error!("Failed to reset stale jobs: {}", e);
        }

        let mut interval_timer = interval(self.check_interval);

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.process_pending_jobs().await {
                log::error!("Error processing sync jobs: {}", e);
            }
        }
    }

    /// Reset jobs that were processing when the server shut down
    async fn reset_stale_jobs(&self) -> Result<(), DbErr> {
        let stale_jobs = platform_sync_job::Entity::find()
            .filter(platform_sync_job::Column::Status.eq(SyncJobStatus::Processing))
            .all(&self.db)
            .await?;

        if !stale_jobs.is_empty() {
            log::info!(
                "Found {} stale processing jobs, resetting to pending",
                stale_jobs.len()
            );

            for job in stale_jobs {
                let mut active_job: platform_sync_job::ActiveModel = job.into();
                active_job.status = Set(SyncJobStatus::Pending);
                active_job.started_at = Set(None);
                active_job.error_message = Set(None); // Clear error for retry

                active_job.update(&self.db).await?;
            }
        }

        Ok(())
    }

    /// Update job progress counts
    async fn update_job_progress(
        &self,
        job_id: Uuid,
        contributions: i32,
        activities: i32,
    ) -> Result<(), DbErr> {
        let job = platform_sync_job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await?;

        if let Some(job_model) = job {
            let mut active_job: platform_sync_job::ActiveModel = job_model.into();
            active_job.contributions_synced = Set(Some(contributions));
            active_job.activities_synced = Set(Some(activities));
            active_job.update(&self.db).await?;
        }

        Ok(())
    }

    /// Update job progress counts with year tracking
    async fn update_job_progress_with_years(
        &self,
        job_id: Uuid,
        contributions: i32,
        activities: i32,
        years_completed: i32,
    ) -> Result<(), DbErr> {
        let job = platform_sync_job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await?;

        if let Some(job_model) = job {
            let mut active_job: platform_sync_job::ActiveModel = job_model.into();
            active_job.contributions_synced = Set(Some(contributions));
            active_job.activities_synced = Set(Some(activities));
            active_job.years_completed = Set(Some(years_completed));
            active_job.update(&self.db).await?;
        }

        Ok(())
    }

    /// Check if a job has been cancelled
    async fn is_job_cancelled(&self, job_id: Uuid) -> Result<bool, DbErr> {
        let job = platform_sync_job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await?;

        if let Some(job_model) = job {
            Ok(job_model.status == SyncJobStatus::Failed
                && job_model
                    .error_message
                    .as_ref()
                    .map(|m| m.contains("Cancelled"))
                    .unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Process all pending jobs
    async fn process_pending_jobs(&self) -> Result<(), DbErr> {
        // Fetch pending jobs ordered by priority
        let pending_jobs = platform_sync_job::Entity::find()
            .filter(platform_sync_job::Column::Status.eq(SyncJobStatus::Pending))
            .order_by_desc(platform_sync_job::Column::Priority)
            .order_by_asc(platform_sync_job::Column::ScheduledAt)
            .limit(5) // Process up to 5 jobs at a time
            .all(&self.db)
            .await?;

        if pending_jobs.is_empty() {
            return Ok(());
        }

        log::info!("Processing {} pending sync jobs", pending_jobs.len());

        for job in pending_jobs {
            if let Err(e) = self.process_job(job).await {
                log::error!("Failed to process sync job: {}", e);
            }
        }

        Ok(())
    }

    /// Process a single job
    async fn process_job(&self, job: platform_sync_job::Model) -> Result<(), anyhow::Error> {
        log::info!(
            "Processing sync job {} for platform account {} (manual: {})",
            job.id,
            job.platform_account_id,
            job.is_manual
        );

        // Mark job as processing
        let mut active_job: platform_sync_job::ActiveModel = job.clone().into();
        active_job.status = Set(SyncJobStatus::Processing);
        active_job.started_at = Set(Some(Utc::now()));
        active_job.error_message = Set(None); // Clear any previous error message

        let processing_job = active_job.update(&self.db).await?;

        // Process the job
        let result = self.execute_sync(&processing_job).await;

        // Update job status
        let mut final_job: platform_sync_job::ActiveModel = processing_job.into();

        match result {
            Ok((contributions_count, activities_count)) => {
                final_job.status = Set(SyncJobStatus::Completed);
                final_job.completed_at = Set(Some(Utc::now()));
                final_job.error_message = Set(None);
                final_job.contributions_synced = Set(Some(contributions_count));
                final_job.activities_synced = Set(Some(activities_count));

                log::info!(
                    "Sync job {} completed successfully: {} contributions, {} activities",
                    job.id,
                    contributions_count,
                    activities_count
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::error!("Sync job {} failed: {}", job.id, error_msg);

                // Check if we should retry
                let retry_count = job.retry_count;
                let max_retries = job.max_retries;

                if retry_count < max_retries {
                    // Retry
                    final_job.status = Set(SyncJobStatus::Pending);
                    final_job.retry_count = Set(retry_count + 1);
                    final_job.error_message = Set(Some(format!(
                        "Attempt {}/{} failed: {}",
                        retry_count + 1,
                        max_retries,
                        error_msg
                    )));
                    final_job.started_at = Set(None);

                    log::info!(
                        "Sync job {} will be retried (attempt {}/{})",
                        job.id,
                        retry_count + 1,
                        max_retries
                    );
                } else {
                    // Max retries reached
                    final_job.status = Set(SyncJobStatus::Failed);
                    final_job.completed_at = Set(Some(Utc::now()));
                    final_job.error_message = Set(Some(error_msg));

                    log::error!("Sync job {} failed after {} attempts", job.id, max_retries);
                }
            }
        }

        final_job.update(&self.db).await?;

        Ok(())
    }

    /// Execute the actual sync
    async fn execute_sync(
        &self,
        job: &platform_sync_job::Model,
    ) -> Result<(i32, i32), anyhow::Error> {
        // Fetch platform account
        let account = git_platform_account::Entity::find_by_id(job.platform_account_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Platform account not found"))?;

        let encrypted_token = account
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No access token found"))?;
        let access_token = decrypt(encrypted_token, &self.encryption_key)?;

        let current_year = Utc::now().year();

        let (start_year, end_year) = if job.sync_all_years {
            (2020, current_year)
        } else if let Some(year) = job.specific_year {
            (year, year)
        } else {
            (current_year, current_year)
        };

        // Calculate total years and update job
        let total_years = (end_year - start_year + 1) as i32;
        let mut job_update: platform_sync_job::ActiveModel = job.clone().into();
        job_update.total_years = Set(Some(total_years));
        job_update.years_completed = Set(Some(0));
        job_update.update(&self.db).await?;

        let mut total_contributions = 0;
        let mut total_activities = 0;

        // Sync contributions if requested
        if job.sync_contributions {
            total_contributions = self
                .sync_contributions(
                    job.id,
                    &account,
                    &access_token,
                    start_year,
                    end_year,
                    current_year,
                )
                .await?;
        }

        // Check if job has been cancelled before starting activities
        if self.is_job_cancelled(job.id).await? {
            log::warn!("🚫 [SyncJob] Job {} cancelled after contributions", job.id);
            return Err(anyhow::anyhow!("Sync cancelled by user"));
        }

        // Sync activities if requested
        if job.sync_activities {
            total_activities = self
                .sync_activities(
                    job.id,
                    job.platform_account_id,
                    start_year,
                    end_year,
                    current_year,
                )
                .await?;
        }

        // Sync profile if requested
        if job.sync_profile {
            self.sync_profile(&account, &access_token).await?;
        }

        Ok((total_contributions, total_activities))
    }

    async fn sync_contributions(
        &self,
        job_id: Uuid,
        account: &git_platform_account::Model,
        access_token: &str,
        start_year: i32,
        end_year: i32,
        current_year: i32,
    ) -> Result<i32, anyhow::Error> {
        let platform_config = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => PlatformConfig::github(),
            git_platform_account::GitPlatform::Gitea => {
                let url = account
                    .platform_url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Gitea URL not found"))?;
                PlatformConfig::gitea_custom(url)
            }
            git_platform_account::GitPlatform::GitLab => {
                let url = account
                    .platform_url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GitLab URL not found"))?;
                PlatformConfig::gitlab_custom(url)
            }
        };

        let client: Box<dyn GitPlatform + Send + Sync> = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => Box::new(GitHubClient::new()),
            git_platform_account::GitPlatform::Gitea => Box::new(GiteaClient::new()),
            git_platform_account::GitPlatform::GitLab => Box::new(GitLabClient::new()),
        };

        let mut all_contributions = Vec::new();
        let mut running_total = 0;

        log::info!(
            "🔄 [SyncJob] Syncing contributions for years {} to {}",
            start_year,
            end_year
        );

        for year in start_year..=end_year {
            // Check if job has been cancelled
            if self.is_job_cancelled(job_id).await? {
                log::warn!("🚫 [SyncJob] Job {} cancelled, stopping sync", job_id);
                return Err(anyhow::anyhow!("Sync cancelled by user"));
            }

            let from_date = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
            let from = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();

            let to_date = if year == current_year {
                Utc::now().date_naive()
            } else {
                chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
            };
            let to = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

            log::info!(
                "🔄 [SyncJob] Fetching year {}: {} to {}",
                year,
                from.format("%Y-%m-%d"),
                to.format("%Y-%m-%d")
            );

            let contributions = client
                .fetch_contributions(
                    &platform_config,
                    &account.platform_username,
                    access_token,
                    from,
                    to,
                )
                .await?;

            log::info!(
                "✅ [SyncJob] Fetched {} contribution days for year {}",
                contributions.len(),
                year
            );

            running_total += contributions.len() as i32;
            all_contributions.extend(contributions);

            // Update job progress after each year (increment years_completed)
            let years_completed = (year - start_year + 1) as i32;
            self.update_job_progress_with_years(job_id, running_total, 0, years_completed)
                .await?;

            // Add delay between years to be extra nice to the API
            if year < end_year {
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }
        }

        // Delete existing contributions in the synced date range
        let delete_from = chrono::NaiveDate::from_ymd_opt(start_year, 1, 1).unwrap();
        let delete_to = if end_year == current_year {
            Utc::now().date_naive()
        } else {
            chrono::NaiveDate::from_ymd_opt(end_year, 12, 31).unwrap()
        };

        log::info!(
            "🗑️  [SyncJob] Deleting existing contributions from {} to {}",
            delete_from,
            delete_to
        );

        contribution::Entity::delete_many()
            .filter(contribution::Column::GitPlatformAccountId.eq(account.id))
            .filter(contribution::Column::ContributionDate.gte(delete_from))
            .filter(contribution::Column::ContributionDate.lte(delete_to))
            .exec(&self.db)
            .await?;

        // Insert new contributions
        let mut total_inserted = 0;

        for contribution_data in all_contributions {
            let contribution = contribution::ActiveModel {
                id: Set(Uuid::new_v4()),
                git_platform_account_id: Set(account.id),
                contribution_date: Set(contribution_data.date),
                count: Set(contribution_data.count),
                repository_name: Set(contribution_data.repository_name),
                is_private_repo: Set(contribution_data.is_private),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            contribution::Entity::insert(contribution)
                .exec(&self.db)
                .await?;

            total_inserted += 1;
        }

        log::info!("✅ [SyncJob] Inserted {} contributions", total_inserted);

        Ok(total_inserted)
    }

    async fn sync_activities(
        &self,
        job_id: Uuid,
        account_id: Uuid,
        start_year: i32,
        end_year: i32,
        current_year: i32,
    ) -> Result<i32, anyhow::Error> {
        let activity_service =
            ActivityAggregationService::new(self.db.clone(), self.encryption_key.clone());

        let from_date = chrono::NaiveDate::from_ymd_opt(start_year, 1, 1).unwrap();
        let to_date = if end_year == current_year {
            Utc::now().date_naive()
        } else {
            chrono::NaiveDate::from_ymd_opt(end_year, 12, 31).unwrap()
        };

        let from = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let to = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        log::info!(
            "📅 [SyncJob] Syncing activities from {} to {}",
            from.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        );

        activity_service
            .sync_single_platform_activity(account_id, from, to)
            .await?;

        // Count activities
        let count = activity::Entity::find()
            .filter(activity::Column::GitPlatformAccountId.eq(account_id))
            .filter(activity::Column::ActivityDate.gte(from.date_naive()))
            .filter(activity::Column::ActivityDate.lte(to.date_naive()))
            .count(&self.db)
            .await? as i32;

        log::info!("✅ [SyncJob] Synced {} activities", count);

        // Update job progress
        if let Ok(Some(job_model)) = platform_sync_job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await
        {
            let contributions = job_model.contributions_synced.unwrap_or(0);
            self.update_job_progress(job_id, contributions, count)
                .await?;
        }

        Ok(count)
    }

    async fn sync_profile(
        &self,
        account: &git_platform_account::Model,
        access_token: &str,
    ) -> Result<(), anyhow::Error> {
        log::info!(
            "👤 [SyncJob] Syncing profile data for {}",
            account.platform_username
        );

        let platform_config = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => PlatformConfig::github(),
            git_platform_account::GitPlatform::Gitea => {
                let url = account
                    .platform_url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Gitea URL not found"))?;
                PlatformConfig::gitea_custom(url)
            }
            git_platform_account::GitPlatform::GitLab => {
                let url = account
                    .platform_url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GitLab URL not found"))?;
                PlatformConfig::gitlab_custom(url)
            }
        };

        // Fetch profile data based on platform type
        let profile_data = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                let client = GitHubClient::new();
                client
                    .fetch_user_profile(&platform_config, &account.platform_username, access_token)
                    .await?
            }
            git_platform_account::GitPlatform::Gitea => {
                let client = GiteaClient::new();
                client
                    .fetch_user_profile(&platform_config, access_token)
                    .await?
            }
            git_platform_account::GitPlatform::GitLab => {
                let client = GitLabClient::new();
                client
                    .fetch_user_profile(&platform_config, access_token)
                    .await?
            }
        };

        // Update account with new profile data
        let mut active_account: git_platform_account::ActiveModel = account.clone().into();

        if let Some(avatar) = profile_data.get("avatar_url").and_then(|v| v.as_str()) {
            active_account.avatar_url = Set(Some(avatar.to_string()));
        }

        if let Some(name) = profile_data.get("name").and_then(|v| v.as_str()) {
            active_account.display_name = Set(Some(name.to_string()));
        }

        if let Some(bio) = profile_data.get("bio").and_then(|v| v.as_str()) {
            active_account.bio = Set(Some(bio.to_string()));
        }

        if let Some(location) = profile_data.get("location").and_then(|v| v.as_str()) {
            active_account.location = Set(Some(location.to_string()));
        }

        if let Some(company) = profile_data.get("company").and_then(|v| v.as_str()) {
            active_account.company = Set(Some(company.to_string()));
        }

        // Platform-specific fields
        match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                if let Some(url) = profile_data.get("html_url").and_then(|v| v.as_str()) {
                    active_account.profile_url = Set(Some(url.to_string()));
                }
                if let Some(followers) = profile_data.get("followers").and_then(|v| v.as_i64()) {
                    active_account.followers_count = Set(Some(followers as i32));
                }
                if let Some(following) = profile_data.get("following").and_then(|v| v.as_i64()) {
                    active_account.following_count = Set(Some(following as i32));
                }
            }
            git_platform_account::GitPlatform::Gitea => {
                if let Some(url) = profile_data.get("html_url").and_then(|v| v.as_str()) {
                    active_account.profile_url = Set(Some(url.to_string()));
                }
                if let Some(followers) =
                    profile_data.get("followers_count").and_then(|v| v.as_i64())
                {
                    active_account.followers_count = Set(Some(followers as i32));
                }
                if let Some(following) =
                    profile_data.get("following_count").and_then(|v| v.as_i64())
                {
                    active_account.following_count = Set(Some(following as i32));
                }
            }
            git_platform_account::GitPlatform::GitLab => {
                if let Some(url) = profile_data.get("web_url").and_then(|v| v.as_str()) {
                    active_account.profile_url = Set(Some(url.to_string()));
                }
                // GitLab doesn't provide followers/following in user API
            }
        }

        active_account.updated_at = Set(Utc::now());
        active_account.update(&self.db).await?;

        log::info!(
            "👤 [SyncJob] Profile synced successfully for {}",
            account.platform_username
        );

        Ok(())
    }
}

/// Start the sync job processor in the background
pub fn start_sync_job_processor(
    db: DatabaseConnection,
    encryption_key: String,
) -> tokio::task::JoinHandle<()> {
    let processor = Arc::new(SyncJobProcessor::new(db, encryption_key, 2)); // Check every 2 seconds

    tokio::spawn(async move {
        processor.start().await;
    })
}
