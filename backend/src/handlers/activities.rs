use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::TimeZone;

use crate::models::{activity, git_platform_account, user_setting};
use crate::services::activity_aggregation::ActivityAggregationService;
use crate::utils::config::Config;

#[derive(Debug, Deserialize)]
pub struct ActivitiesQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: String,
    pub activity_type: String,
    pub date: String,
    pub metadata: serde_json::Value,
    pub repository_name: Option<String>,
    pub repository_url: Option<String>,
    pub is_private: bool,
    pub count: i32,
    pub primary_language: Option<String>,
    pub organization_name: Option<String>,
    pub organization_avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActivitiesResponse {
    pub activities: Vec<ActivityResponse>,
    pub total: i32,
    pub has_more: bool,
}

/// GET /api/activities
/// Get user's activity timeline
pub async fn get_activities(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<ActivitiesQuery>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Get user settings for privacy filtering
    let settings = user_setting::Entity::find()
        .filter(user_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let show_private_contributions = settings
        .as_ref()
        .map(|s| s.show_private_contributions)
        .unwrap_or(true);

    // Get all active platform accounts for this user
    let mut accounts_query = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true));

    // Filter by platform type if specified
    if let Some(platform_filter) = &query.platform {
        let platform_type = match platform_filter.to_lowercase().as_str() {
            "github" => git_platform_account::GitPlatform::GitHub,
            "gitea" => git_platform_account::GitPlatform::Gitea,
            "gitlab" => git_platform_account::GitPlatform::GitLab,
            _ => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid platform filter: {}", platform_filter)
                })));
            }
        };
        accounts_query = accounts_query.filter(git_platform_account::Column::PlatformType.eq(platform_type));
    }

    let accounts = accounts_query.all(db.as_ref()).await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if accounts.is_empty() {
        return Ok(HttpResponse::Ok().json(ActivitiesResponse {
            activities: vec![],
            total: 0,
            has_more: false,
        }));
    }

    let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

    // Build query for activities
    let mut activity_query = activity::Entity::find()
        .filter(activity::Column::GitPlatformAccountId.is_in(account_ids))
        .order_by_desc(activity::Column::ActivityDate);

    // Apply privacy filter
    if !show_private_contributions {
        activity_query = activity_query.filter(activity::Column::IsPrivateRepo.eq(false));
    }

    // Apply date range if provided
    if let Some(from_str) = &query.from {
        if let Ok(from_date) = chrono::NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.gte(from_date));
        }
    }

    if let Some(to_str) = &query.to {
        if let Ok(to_date) = chrono::NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
            activity_query = activity_query.filter(activity::Column::ActivityDate.lte(to_date));
        }
    }

    // Count total activities before pagination
    let total = activity_query
        .clone()
        .count(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Apply pagination
    let limit = query.limit.unwrap_or(50).min(100) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;

    activity_query = activity_query.limit(limit).offset(offset);

    let activities = activity_query.all(db.as_ref()).await.map_err(|e| {
        log::error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let has_more = (offset + limit) < total;

    let activity_responses: Vec<ActivityResponse> = activities
        .into_iter()
        .map(|a| ActivityResponse {
            id: a.id.to_string(),
            activity_type: format!("{:?}", a.activity_type),
            date: a.activity_date.format("%Y-%m-%d").to_string(),
            metadata: a.metadata,
            repository_name: a.repository_name,
            repository_url: a.repository_url,
            is_private: a.is_private_repo,
            count: a.count,
            primary_language: a.primary_language,
            organization_name: a.organization_name,
            organization_avatar_url: a.organization_avatar_url,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ActivitiesResponse {
        activities: activity_responses,
        total: total as i32,
        has_more,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SyncActivitiesQuery {
    pub year: Option<i32>,
    pub all_years: Option<bool>,
    pub platform_account_id: Option<String>,
}

/// POST /api/activities/sync
/// Trigger manual sync of activities
pub async fn sync_activities(
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<SyncActivitiesQuery>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let aggregation_service = ActivityAggregationService::new(
        db.get_ref().clone(),
        config.encryption_key.clone(),
    );

    // Determine date range based on query parameters
    let (from, to) = if query.all_years.unwrap_or(false) {
        // Sync from 2020 to now
        let from = chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).single()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid date"))?;
        let to = chrono::Utc::now();
        (from, to)
    } else if let Some(year) = query.year {
        // Sync specific year
        let from = chrono::Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid year"))?;
        let to = chrono::Utc.with_ymd_and_hms(year, 12, 31, 23, 59, 59).single()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid year"))?;
        (from, to)
    } else {
        // Default: current year
        let now = chrono::Utc::now();
        let current_year = now.format("%Y").to_string().parse::<i32>().unwrap();
        let from = chrono::Utc.with_ymd_and_hms(current_year, 1, 1, 0, 0, 0).single()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid date"))?;
        let to = now;
        (from, to)
    };

    log::info!("Syncing activities from {} to {}", from, to);

    // If platform_account_id is provided, sync only that specific platform
    if let Some(account_id_str) = &query.platform_account_id {
        let account_id = Uuid::parse_str(account_id_str).map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("Invalid platform account ID: {}", e))
        })?;

        // Verify the account belongs to this user
        let account = git_platform_account::Entity::find_by_id(account_id)
            .one(db.as_ref())
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
            .ok_or_else(|| actix_web::error::ErrorNotFound("Platform account not found"))?;

        if account.user_id != user_id {
            return Err(actix_web::error::ErrorForbidden("Not authorized"));
        }

        log::info!("Syncing activities for single platform account: {}", account_id);

        aggregation_service
            .sync_single_platform_activity(account_id, from, to)
            .await
            .map_err(|e| {
                log::error!("Failed to sync activities for platform {}: {}", account_id, e);
                actix_web::error::ErrorInternalServerError(format!("Failed to sync activities: {}", e))
            })?;
    } else {
        // Sync all platform accounts for the user
        log::info!("Syncing activities for all platform accounts");

        aggregation_service
            .sync_user_activities(user_id, from, to)
            .await
            .map_err(|e| {
                log::error!("Failed to sync activities: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to sync activities")
            })?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Activities synced successfully"
    })))
}
