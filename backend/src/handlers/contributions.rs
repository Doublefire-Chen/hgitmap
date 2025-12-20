use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{contribution, git_platform_account, user_setting};

#[derive(Debug, Deserialize)]
pub struct ContributionsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContributionDay {
    pub date: String,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct ContributionsResponse {
    pub contributions: Vec<ContributionDay>,
    pub total_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ContributionStatsResponse {
    pub total_contributions: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub active_platforms: i32,
}

/// GET /api/contributions
/// Get aggregated contribution data for heatmap
pub async fn get_contributions(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<ContributionsQuery>,
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
    let accounts = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if accounts.is_empty() {
        return Ok(HttpResponse::Ok().json(ContributionsResponse {
            contributions: vec![],
            total_count: 0,
        }));
    }

    let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

    // Build query for contributions
    let mut contribution_query = contribution::Entity::find()
        .filter(contribution::Column::GitPlatformAccountId.is_in(account_ids));

    // Apply privacy filter
    if !show_private_contributions {
        contribution_query =
            contribution_query.filter(contribution::Column::IsPrivateRepo.eq(false));
    }

    // Apply date range if provided
    if let Some(from_str) = &query.from {
        if let Ok(from_date) = chrono::NaiveDate::parse_from_str(from_str, "%Y-%m-%d") {
            contribution_query =
                contribution_query.filter(contribution::Column::ContributionDate.gte(from_date));
        }
    }

    if let Some(to_str) = &query.to {
        if let Ok(to_date) = chrono::NaiveDate::parse_from_str(to_str, "%Y-%m-%d") {
            contribution_query =
                contribution_query.filter(contribution::Column::ContributionDate.lte(to_date));
        }
    }

    let contributions = contribution_query
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Aggregate contributions by date
    let mut contribution_map: HashMap<chrono::NaiveDate, i32> = HashMap::new();
    for contrib in contributions {
        *contribution_map.entry(contrib.contribution_date).or_insert(0) += contrib.count;
    }

    let mut contribution_days: Vec<ContributionDay> = contribution_map
        .iter()
        .map(|(date, count)| ContributionDay {
            date: date.format("%Y-%m-%d").to_string(),
            count: *count,
        })
        .collect();

    contribution_days.sort_by(|a, b| a.date.cmp(&b.date));

    let total_count: i32 = contribution_days.iter().map(|c| c.count).sum();

    Ok(HttpResponse::Ok().json(ContributionsResponse {
        contributions: contribution_days,
        total_count,
    }))
}

/// GET /api/contributions/stats
/// Get contribution statistics
pub async fn get_stats(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
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

    // Get all active platform accounts
    let accounts = git_platform_account::Entity::find()
        .filter(git_platform_account::Column::UserId.eq(user_id))
        .filter(git_platform_account::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let active_platforms = accounts.len() as i32;

    if accounts.is_empty() {
        return Ok(HttpResponse::Ok().json(ContributionStatsResponse {
            total_contributions: 0,
            current_streak: 0,
            longest_streak: 0,
            active_platforms: 0,
        }));
    }

    let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

    // Get contributions
    let mut contribution_query = contribution::Entity::find()
        .filter(contribution::Column::GitPlatformAccountId.is_in(account_ids))
        .order_by_asc(contribution::Column::ContributionDate);

    if !show_private_contributions {
        contribution_query =
            contribution_query.filter(contribution::Column::IsPrivateRepo.eq(false));
    }

    let contributions = contribution_query
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Calculate stats
    let total_contributions: i32 = contributions.iter().map(|c| c.count).sum();

    // Calculate streaks
    let (current_streak, longest_streak) = calculate_streaks(&contributions);

    Ok(HttpResponse::Ok().json(ContributionStatsResponse {
        total_contributions,
        current_streak,
        longest_streak,
        active_platforms,
    }))
}

/// Calculate current and longest contribution streaks
fn calculate_streaks(contributions: &[contribution::Model]) -> (i32, i32) {
    if contributions.is_empty() {
        return (0, 0);
    }

    // Group by date and sum counts
    let mut contribution_map: HashMap<chrono::NaiveDate, i32> = HashMap::new();
    for contrib in contributions {
        *contribution_map.entry(contrib.contribution_date).or_insert(0) += contrib.count;
    }

    let mut dates: Vec<chrono::NaiveDate> = contribution_map.keys().copied().collect();
    dates.sort();

    let today = chrono::Utc::now().date_naive();
    let mut current_streak = 0;
    let mut longest_streak = 0;
    let mut temp_streak = 0;
    let mut last_date: Option<chrono::NaiveDate> = None;

    for date in dates.iter().rev() {
        if let Some(prev_date) = last_date {
            if (*date + chrono::Duration::days(1)) == prev_date {
                temp_streak += 1;
            } else {
                longest_streak = longest_streak.max(temp_streak);
                temp_streak = 1;
            }
        } else {
            temp_streak = 1;
        }
        last_date = Some(*date);
    }

    longest_streak = longest_streak.max(temp_streak);

    // Calculate current streak (must include today or yesterday)
    if let Some(&last_date) = dates.last() {
        if last_date == today || last_date == today - chrono::Duration::days(1) {
            current_streak = 1;
            let mut check_date = last_date - chrono::Duration::days(1);
            while contribution_map.contains_key(&check_date) {
                current_streak += 1;
                check_date = check_date - chrono::Duration::days(1);
            }
        }
    }

    (current_streak, longest_streak)
}
