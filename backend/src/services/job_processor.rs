use sea_orm::*;
use tokio::time::{interval, Duration};
use chrono::Utc;
use std::sync::Arc;

use crate::models::{
    heatmap_generation_job::{self, GenerationJobStatus},
    heatmap_theme,
    heatmap_generation_setting,
};
use crate::services::heatmap_generator::HeatmapGenerator;

pub struct JobProcessor {
    db: DatabaseConnection,
    check_interval: Duration,
}

impl JobProcessor {
    pub fn new(db: DatabaseConnection, check_interval_secs: u64) -> Self {
        Self {
            db,
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    /// Start the job processor loop
    pub async fn start(self: Arc<Self>) {
        log::info!("Starting heatmap generation job processor");

        let mut interval = interval(self.check_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.process_pending_jobs().await {
                log::error!("Error processing jobs: {}", e);
            }

            if let Err(e) = self.schedule_automatic_jobs().await {
                log::error!("Error scheduling automatic jobs: {}", e);
            }
        }
    }

    /// Process all pending jobs
    async fn process_pending_jobs(&self) -> Result<(), DbErr> {
        // Fetch pending jobs ordered by priority
        let pending_jobs = heatmap_generation_job::Entity::find()
            .filter(heatmap_generation_job::Column::Status.eq(GenerationJobStatus::Pending))
            .order_by_desc(heatmap_generation_job::Column::Priority)
            .order_by_asc(heatmap_generation_job::Column::ScheduledAt)
            .limit(10) // Process up to 10 jobs at a time
            .all(&self.db)
            .await?;

        if pending_jobs.is_empty() {
            return Ok(());
        }

        log::info!("Processing {} pending generation jobs", pending_jobs.len());

        for job in pending_jobs {
            if let Err(e) = self.process_job(job).await {
                log::error!("Failed to process job: {}", e);
            }
        }

        Ok(())
    }

    /// Process a single job
    async fn process_job(&self, job: heatmap_generation_job::Model) -> Result<(), anyhow::Error> {
        log::info!(
            "Processing job {} for user {} (manual: {})",
            job.id,
            job.user_id,
            job.is_manual
        );

        // Mark job as processing
        let mut active_job: heatmap_generation_job::ActiveModel = job.clone().into();
        active_job.status = Set(GenerationJobStatus::Processing);
        active_job.started_at = Set(Some(Utc::now()));

        let processing_job = active_job.update(&self.db).await?;

        // Process the job
        let result = self.execute_generation(&processing_job).await;

        // Update job status
        let mut final_job: heatmap_generation_job::ActiveModel = processing_job.into();

        match result {
            Ok(_) => {
                final_job.status = Set(GenerationJobStatus::Completed);
                final_job.completed_at = Set(Some(Utc::now()));
                final_job.error_message = Set(None);

                log::info!("Job {} completed successfully", job.id);
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::error!("Job {} failed: {}", job.id, error_msg);

                // Check if we should retry
                let retry_count = job.retry_count;
                let max_retries = job.max_retries;

                if retry_count < max_retries {
                    // Retry
                    final_job.status = Set(GenerationJobStatus::Pending);
                    final_job.retry_count = Set(retry_count + 1);
                    final_job.error_message = Set(Some(format!(
                        "Attempt {}/{} failed: {}",
                        retry_count + 1,
                        max_retries,
                        error_msg
                    )));
                    final_job.started_at = Set(None);

                    log::info!("Job {} will be retried (attempt {}/{})", job.id, retry_count + 1, max_retries);
                } else {
                    // Max retries reached
                    final_job.status = Set(GenerationJobStatus::Failed);
                    final_job.completed_at = Set(Some(Utc::now()));
                    final_job.error_message = Set(Some(error_msg));

                    log::error!("Job {} failed after {} attempts", job.id, max_retries);
                }
            }
        }

        final_job.update(&self.db).await?;

        Ok(())
    }

    /// Execute the actual generation
    async fn execute_generation(
        &self,
        job: &heatmap_generation_job::Model,
    ) -> Result<(), anyhow::Error> {
        let generator = HeatmapGenerator::new(self.db.clone());

        if let Some(theme_id) = job.theme_id {
            // Generate for specific theme
            let theme = heatmap_theme::Entity::find_by_id(theme_id)
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Theme not found"))?;

            generator.generate_for_theme(job.user_id, &theme).await?;
        } else {
            // Generate for all themes
            let themes = heatmap_theme::Entity::find()
                .filter(heatmap_theme::Column::UserId.eq(job.user_id))
                .all(&self.db)
                .await?;

            for theme in themes {
                generator.generate_for_theme(job.user_id, &theme).await?;
            }
        }

        Ok(())
    }

    /// Schedule automatic generation jobs based on user settings
    async fn schedule_automatic_jobs(&self) -> Result<(), DbErr> {
        // Find users who need automatic generation
        let settings = heatmap_generation_setting::Entity::find()
            .filter(heatmap_generation_setting::Column::AutoGenerationEnabled.eq(true))
            .all(&self.db)
            .await?;

        let now = Utc::now();

        for setting in settings {
            // Check if it's time to generate
            let should_generate = match setting.next_scheduled_generation_at {
                Some(next_time) => now >= next_time,
                None => true, // First time, generate now
            };

            if !should_generate {
                continue;
            }

            // Check if there's already a pending job for this user
            let existing_job = heatmap_generation_job::Entity::find()
                .filter(heatmap_generation_job::Column::UserId.eq(setting.user_id))
                .filter(heatmap_generation_job::Column::Status.eq(GenerationJobStatus::Pending))
                .filter(heatmap_generation_job::Column::IsManual.eq(false))
                .one(&self.db)
                .await?;

            if existing_job.is_some() {
                continue; // Already has a pending job
            }

            // Create new job
            let job = heatmap_generation_job::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                user_id: Set(setting.user_id),
                theme_id: Set(None), // Generate all themes
                status: Set(GenerationJobStatus::Pending),
                scheduled_at: Set(now),
                started_at: Set(None),
                completed_at: Set(None),
                error_message: Set(None),
                retry_count: Set(0),
                max_retries: Set(3),
                is_manual: Set(false),
                priority: Set(0), // Normal priority for automatic jobs
                created_at: Set(now),
            };

            heatmap_generation_job::Entity::insert(job)
                .exec(&self.db)
                .await?;

            // Update next scheduled time
            let next_time = now + chrono::Duration::minutes(setting.update_interval_minutes as i64);
            let user_id = setting.user_id;

            let mut active_setting: heatmap_generation_setting::ActiveModel = setting.into();
            active_setting.last_scheduled_generation_at = Set(Some(now));
            active_setting.next_scheduled_generation_at = Set(Some(next_time));
            active_setting.update(&self.db).await?;

            log::info!(
                "Scheduled automatic generation job for user {} (next: {})",
                user_id,
                next_time
            );
        }

        Ok(())
    }

    /// Clean up old completed jobs (optional maintenance task)
    #[allow(dead_code)]
    pub async fn cleanup_old_jobs(&self, days_to_keep: i64) -> Result<(), DbErr> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);

        let result = heatmap_generation_job::Entity::delete_many()
            .filter(heatmap_generation_job::Column::Status.eq(GenerationJobStatus::Completed))
            .filter(heatmap_generation_job::Column::CompletedAt.lt(cutoff_date))
            .exec(&self.db)
            .await?;

        if result.rows_affected > 0 {
            log::info!("Cleaned up {} old completed jobs", result.rows_affected);
        }

        Ok(())
    }
}

/// Start the job processor in the background
pub fn start_job_processor(db: DatabaseConnection) -> tokio::task::JoinHandle<()> {
    let processor = Arc::new(JobProcessor::new(db, 30)); // Check every 30 seconds

    tokio::spawn(async move {
        processor.start().await;
    })
}
