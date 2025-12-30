use actix_web::{web, HttpResponse, Responder};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{git_platform_account, platform_sync_job};

#[derive(Debug, Deserialize, Serialize)]
pub struct SyncJobResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

/// POST /api/platforms/:id/sync-async?all_years=true
/// Create an async sync job for a platform account (new non-blocking endpoint)
pub async fn sync_platform_async(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    let account_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid account ID: {}", e)))?;

    // Find the account
    let account = git_platform_account::Entity::find_by_id(account_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Account not found"))?;

    // Verify ownership
    if account.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    if !account.is_active {
        return Err(actix_web::error::ErrorBadRequest("Account is not active"));
    }

    log::info!(
        "🔄 [Sync] Creating async sync job for account: {}",
        account_id
    );
    log::info!(
        "🔄 [Sync] Platform: {:?}, Username: {}",
        account.platform_type,
        account.platform_username
    );

    // Check if this is profile-only sync
    let profile_only = query
        .get("profile_only")
        .map(|v| v == "true")
        .unwrap_or(false);

    // Check sync mode: all_years, specific year, or current year (default)
    let sync_all_years = query.get("all_years").map(|v| v == "true").unwrap_or(false);

    let specific_year = query.get("year").and_then(|v| v.parse::<i32>().ok());

    // Determine what to sync
    let sync_contributions = if profile_only {
        false
    } else {
        account.sync_contributions
    };

    let sync_activities = if profile_only {
        false
    } else {
        account.sync_contributions // Activities sync when contributions are enabled
    };

    let sync_profile = profile_only || account.sync_profile;

    // Check if there's already a pending or processing job for this account
    let existing_job = platform_sync_job::Entity::find()
        .filter(platform_sync_job::Column::PlatformAccountId.eq(account_id))
        .filter(platform_sync_job::Column::Status.is_in([
            platform_sync_job::SyncJobStatus::Pending,
            platform_sync_job::SyncJobStatus::Processing,
        ]))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(job) = existing_job {
        log::info!("⏳ [Sync] Job already exists for this account: {}", job.id);
        return Ok(HttpResponse::Accepted().json(SyncJobResponse {
            job_id: job.id.to_string(),
            status: format!("{:?}", job.status).to_lowercase(),
            message:
                "A sync job is already running for this account. Please wait for it to complete."
                    .to_string(),
        }));
    }

    // Create a new sync job
    let job_id = Uuid::new_v4();
    let job = platform_sync_job::ActiveModel {
        id: Set(job_id),
        user_id: Set(user_id),
        platform_account_id: Set(account_id),
        status: Set(platform_sync_job::SyncJobStatus::Pending),
        sync_all_years: Set(sync_all_years),
        specific_year: Set(specific_year),
        sync_contributions: Set(sync_contributions),
        sync_activities: Set(sync_activities),
        sync_profile: Set(sync_profile),
        scheduled_at: Set(chrono::Utc::now()),
        started_at: Set(None),
        completed_at: Set(None),
        error_message: Set(None),
        retry_count: Set(0),
        max_retries: Set(3),
        contributions_synced: Set(None),
        activities_synced: Set(None),
        years_completed: Set(None),
        total_years: Set(None),
        is_manual: Set(true),
        priority: Set(10), // Higher priority for manual triggers
        created_at: Set(chrono::Utc::now()),
    };

    platform_sync_job::Entity::insert(job)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to create sync job: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create sync job")
        })?;

    log::info!("✅ [Sync] Created async sync job: {}", job_id);

    let sync_type = if profile_only {
        "profile"
    } else if sync_all_years {
        "all years (2020-present)"
    } else if let Some(year) = specific_year {
        return Ok(HttpResponse::Accepted().json(SyncJobResponse {
            job_id: job_id.to_string(),
            status: "pending".to_string(),
            message: format!(
                "Sync job created for year {}. The sync will run in the background.",
                year
            ),
        }));
    } else {
        "current year"
    };

    Ok(HttpResponse::Accepted().json(SyncJobResponse {
        job_id: job_id.to_string(),
        status: "pending".to_string(),
        message: format!("Sync job created for {}. The sync will run in the background and may take several minutes.", sync_type),
    }))
}

/// GET /api/platforms/sync-jobs/:job_id
/// Get the status of a sync job
pub async fn get_sync_job_status(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    let job_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid job ID: {}", e)))?;

    let job = platform_sync_job::Entity::find_by_id(job_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Job not found"))?;

    // Verify ownership
    if job.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    #[derive(Serialize)]
    struct JobStatusResponse {
        job_id: String,
        status: String,
        scheduled_at: String,
        started_at: Option<String>,
        completed_at: Option<String>,
        error_message: Option<String>,
        retry_count: i32,
        contributions_synced: Option<i32>,
        activities_synced: Option<i32>,
    }

    Ok(HttpResponse::Ok().json(JobStatusResponse {
        job_id: job.id.to_string(),
        status: format!("{:?}", job.status).to_lowercase(),
        scheduled_at: job.scheduled_at.to_rfc3339(),
        started_at: job.started_at.map(|t| t.to_rfc3339()),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
        error_message: job.error_message,
        retry_count: job.retry_count,
        contributions_synced: job.contributions_synced,
        activities_synced: job.activities_synced,
    }))
}

/// GET /api/platforms/sync-jobs?status=pending&limit=50
/// List sync jobs for the current user
pub async fn list_sync_jobs(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    use sea_orm::QueryOrder;

    let mut query_builder =
        platform_sync_job::Entity::find().filter(platform_sync_job::Column::UserId.eq(user_id));

    // Filter by status if provided
    if let Some(status_str) = query.get("status") {
        let status = match status_str.as_str() {
            "pending" => platform_sync_job::SyncJobStatus::Pending,
            "processing" => platform_sync_job::SyncJobStatus::Processing,
            "completed" => platform_sync_job::SyncJobStatus::Completed,
            "failed" => platform_sync_job::SyncJobStatus::Failed,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
        };
        query_builder = query_builder.filter(platform_sync_job::Column::Status.eq(status));
    }

    // Limit results
    let limit = query
        .get("limit")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(50)
        .min(100); // Max 100 jobs

    let jobs = query_builder
        .order_by_desc(platform_sync_job::Column::ScheduledAt)
        .limit(limit)
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    #[derive(Serialize)]
    struct JobListItem {
        id: String,
        platform_account_id: String,
        status: String,
        sync_all_years: bool,
        specific_year: Option<i32>,
        sync_contributions: bool,
        sync_activities: bool,
        sync_profile: bool,
        scheduled_at: String,
        started_at: Option<String>,
        completed_at: Option<String>,
        error_message: Option<String>,
        retry_count: i32,
        contributions_synced: Option<i32>,
        activities_synced: Option<i32>,
        years_completed: Option<i32>,
        total_years: Option<i32>,
    }

    let response: Vec<JobListItem> = jobs
        .into_iter()
        .map(|job| JobListItem {
            id: job.id.to_string(),
            platform_account_id: job.platform_account_id.to_string(),
            status: format!("{:?}", job.status).to_lowercase(),
            sync_all_years: job.sync_all_years,
            specific_year: job.specific_year,
            sync_contributions: job.sync_contributions,
            sync_activities: job.sync_activities,
            sync_profile: job.sync_profile,
            scheduled_at: job.scheduled_at.to_rfc3339(),
            started_at: job.started_at.map(|t| t.to_rfc3339()),
            completed_at: job.completed_at.map(|t| t.to_rfc3339()),
            error_message: job.error_message,
            retry_count: job.retry_count,
            contributions_synced: job.contributions_synced,
            activities_synced: job.activities_synced,
            years_completed: job.years_completed,
            total_years: job.total_years,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/platforms/sync-jobs/:id
/// Cancel a pending or processing sync job
pub async fn cancel_sync_job(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    let job_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid job ID: {}", e)))?;

    // Find the job
    let job = platform_sync_job::Entity::find_by_id(job_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Sync job not found"))?;

    // Verify ownership
    if job.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    // Only allow cancelling pending or processing jobs
    match job.status {
        platform_sync_job::SyncJobStatus::Pending
        | platform_sync_job::SyncJobStatus::Processing => {
            // Update job status to failed with cancellation message
            let mut active_job: platform_sync_job::ActiveModel = job.into();
            active_job.status = Set(platform_sync_job::SyncJobStatus::Failed);
            active_job.completed_at = Set(Some(chrono::Utc::now()));
            active_job.error_message = Set(Some("Cancelled by user".to_string()));

            active_job.update(db.as_ref()).await.map_err(|e| {
                log::error!("Failed to cancel sync job: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to cancel job")
            })?;

            log::info!("🚫 Sync job {} cancelled by user", job_id);

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Sync job cancelled successfully"
            })))
        }
        _ => Err(actix_web::error::ErrorBadRequest(
            "Can only cancel pending or processing jobs",
        )),
    }
}

/// DELETE /api/platforms/sync-jobs/:id/delete
/// Delete a completed or failed sync job
pub async fn delete_sync_job(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    let job_id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid job ID: {}", e)))?;

    // Find the job
    let job = platform_sync_job::Entity::find_by_id(job_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Sync job not found"))?;

    // Verify ownership
    if job.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Not authorized"));
    }

    // Only allow deleting completed or failed jobs
    match job.status {
        platform_sync_job::SyncJobStatus::Completed | platform_sync_job::SyncJobStatus::Failed => {
            platform_sync_job::Entity::delete_by_id(job_id)
                .exec(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Failed to delete sync job: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to delete job")
                })?;

            log::info!("🗑️  Sync job {} deleted by user", job_id);

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Sync job deleted successfully"
            })))
        }
        _ => Err(actix_web::error::ErrorBadRequest(
            "Can only delete completed or failed jobs",
        )),
    }
}
