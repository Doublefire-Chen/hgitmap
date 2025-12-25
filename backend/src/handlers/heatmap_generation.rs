use actix_web::{web, HttpResponse, Responder};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::models::{heatmap_generation_setting, heatmap_generation_job, generated_heatmap, heatmap_theme, user, git_platform_account};
use crate::services::heatmap_generator::HeatmapGenerator;

// ============ Request/Response DTOs ============

#[derive(Debug, Serialize)]
pub struct GenerationSettingsResponse {
    pub update_interval_minutes: i32,
    pub auto_generation_enabled: bool,
    pub date_range_days: i32,
    pub include_private_contributions: bool,
    pub storage_path: Option<String>,
    pub last_scheduled_generation_at: Option<String>,
    pub next_scheduled_generation_at: Option<String>,
    pub updated_at: String,
}

impl From<heatmap_generation_setting::Model> for GenerationSettingsResponse {
    fn from(model: heatmap_generation_setting::Model) -> Self {
        Self {
            update_interval_minutes: model.update_interval_minutes,
            auto_generation_enabled: model.auto_generation_enabled,
            date_range_days: model.date_range_days,
            include_private_contributions: model.include_private_contributions,
            storage_path: model.storage_path,
            last_scheduled_generation_at: model.last_scheduled_generation_at.map(|dt| dt.to_rfc3339()),
            next_scheduled_generation_at: model.next_scheduled_generation_at.map(|dt| dt.to_rfc3339()),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateGenerationSettingsRequest {
    pub update_interval_minutes: Option<i32>,
    pub auto_generation_enabled: Option<bool>,
    pub date_range_days: Option<i32>,
    pub include_private_contributions: Option<bool>,
    pub storage_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GeneratedHeatmapInfo {
    pub id: String,
    pub theme_id: String,
    pub format: String,
    pub file_path: String,
    pub file_size_bytes: Option<i64>,
    pub contribution_count: i32,
    pub date_range_start: String,
    pub date_range_end: String,
    pub generated_at: String,
    pub is_valid: bool,
    pub access_count: i32,
}

impl From<generated_heatmap::Model> for GeneratedHeatmapInfo {
    fn from(model: generated_heatmap::Model) -> Self {
        let format_str = match model.format {
            crate::models::heatmap_theme::HeatmapFormat::Svg => "svg",
            crate::models::heatmap_theme::HeatmapFormat::Png => "png",
            crate::models::heatmap_theme::HeatmapFormat::Jpeg => "jpeg",
            crate::models::heatmap_theme::HeatmapFormat::WebP => "webp",
        };

        Self {
            id: model.id.to_string(),
            theme_id: model.theme_id.to_string(),
            format: format_str.to_string(),
            file_path: model.file_path,
            file_size_bytes: model.file_size_bytes,
            contribution_count: model.contribution_count,
            date_range_start: model.date_range_start.to_string(),
            date_range_end: model.date_range_end.to_string(),
            generated_at: model.generated_at.to_rfc3339(),
            is_valid: model.is_valid,
            access_count: model.access_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GenerationJobResponse {
    pub id: String,
    pub theme_id: Option<String>,
    pub status: String,
    pub scheduled_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub is_manual: bool,
}

impl From<heatmap_generation_job::Model> for GenerationJobResponse {
    fn from(model: heatmap_generation_job::Model) -> Self {
        Self {
            id: model.id.to_string(),
            theme_id: model.theme_id.map(|id| id.to_string()),
            status: match model.status {
                heatmap_generation_job::GenerationJobStatus::Pending => "pending".to_string(),
                heatmap_generation_job::GenerationJobStatus::Processing => "processing".to_string(),
                heatmap_generation_job::GenerationJobStatus::Completed => "completed".to_string(),
                heatmap_generation_job::GenerationJobStatus::Failed => "failed".to_string(),
            },
            scheduled_at: model.scheduled_at.to_rfc3339(),
            started_at: model.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: model.completed_at.map(|dt| dt.to_rfc3339()),
            error_message: model.error_message,
            is_manual: model.is_manual,
        }
    }
}

// ============ Generation Settings Handlers ============

/// GET /api/heatmap/settings
/// Get generation settings
pub async fn get_generation_settings(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let settings = heatmap_generation_setting::Entity::find()
        .filter(heatmap_generation_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    match settings {
        Some(settings) => Ok(HttpResponse::Ok().json(GenerationSettingsResponse::from(settings))),
        None => {
            // Create default settings
            let new_settings = heatmap_generation_setting::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                update_interval_minutes: Set(60),
                auto_generation_enabled: Set(false),
                date_range_days: Set(365),
                include_private_contributions: Set(true),
                storage_path: Set(None),
                last_scheduled_generation_at: Set(None),
                next_scheduled_generation_at: Set(None),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
            };

            let settings = heatmap_generation_setting::Entity::insert(new_settings)
                .exec_with_returning(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Failed to create settings: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to create settings")
                })?;

            Ok(HttpResponse::Ok().json(GenerationSettingsResponse::from(settings)))
        }
    }
}

/// PUT /api/heatmap/settings
/// Update generation settings
pub async fn update_generation_settings(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<UpdateGenerationSettingsRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Validate update_interval_minutes if provided
    if let Some(interval) = payload.update_interval_minutes {
        if interval < 15 || interval > 1440 {
            return Err(actix_web::error::ErrorBadRequest(
                "Update interval must be between 15 and 1440 minutes"
            ));
        }
    }

    // Validate date_range_days if provided
    if let Some(days) = payload.date_range_days {
        if days < 1 || days > 730 {
            return Err(actix_web::error::ErrorBadRequest(
                "Date range must be between 1 and 730 days"
            ));
        }
    }

    // Validate that user has at least one platform with at least one sync type enabled
    // when trying to enable auto sync
    if let Some(enabled) = payload.auto_generation_enabled {
        if enabled {
            let platforms = git_platform_account::Entity::find()
                .filter(git_platform_account::Column::UserId.eq(user_id))
                .filter(git_platform_account::Column::IsActive.eq(true))
                .all(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?;

            let has_any_sync_enabled = platforms.iter().any(|p| {
                p.sync_profile || p.sync_contributions
            });

            if !has_any_sync_enabled {
                return Err(actix_web::error::ErrorBadRequest(
                    "Cannot enable automatic sync: At least one platform must have at least one sync type enabled (Profile or Heatmap+Activities)"
                ));
            }
        }
    }

    let settings = heatmap_generation_setting::Entity::find()
        .filter(heatmap_generation_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let updated_settings = match settings {
        Some(settings) => {
            let mut active_settings: heatmap_generation_setting::ActiveModel = settings.into();

            if let Some(interval) = payload.update_interval_minutes {
                active_settings.update_interval_minutes = Set(interval);
            }

            if let Some(enabled) = payload.auto_generation_enabled {
                active_settings.auto_generation_enabled = Set(enabled);
            }

            if let Some(days) = payload.date_range_days {
                active_settings.date_range_days = Set(days);
            }

            if let Some(include_private) = payload.include_private_contributions {
                active_settings.include_private_contributions = Set(include_private);
            }

            if let Some(path) = &payload.storage_path {
                active_settings.storage_path = Set(Some(path.clone()));
            }

            active_settings.updated_at = Set(chrono::Utc::now());

            active_settings.update(db.as_ref()).await.map_err(|e| {
                log::error!("Failed to update settings: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to update settings")
            })?
        }
        None => {
            // Create new settings
            let new_settings = heatmap_generation_setting::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                update_interval_minutes: Set(payload.update_interval_minutes.unwrap_or(60)),
                auto_generation_enabled: Set(payload.auto_generation_enabled.unwrap_or(false)),
                date_range_days: Set(payload.date_range_days.unwrap_or(365)),
                include_private_contributions: Set(payload.include_private_contributions.unwrap_or(true)),
                storage_path: Set(payload.storage_path.clone()),
                last_scheduled_generation_at: Set(None),
                next_scheduled_generation_at: Set(None),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
            };

            heatmap_generation_setting::Entity::insert(new_settings)
                .exec_with_returning(db.as_ref())
                .await
                .map_err(|e| {
                    log::error!("Failed to create settings: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to create settings")
                })?
        }
    };

    Ok(HttpResponse::Ok().json(GenerationSettingsResponse::from(updated_settings)))
}

// ============ Manual Generation Handlers ============

/// POST /api/heatmap/generate
/// Manually trigger heatmap generation for all themes
pub async fn trigger_generation(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Create a generation job
    let job = heatmap_generation_job::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        theme_id: Set(None), // None means all themes
        status: Set(heatmap_generation_job::GenerationJobStatus::Pending),
        scheduled_at: Set(chrono::Utc::now()),
        started_at: Set(None),
        completed_at: Set(None),
        error_message: Set(None),
        retry_count: Set(0),
        max_retries: Set(3),
        is_manual: Set(true),
        priority: Set(10), // Higher priority for manual triggers
        created_at: Set(chrono::Utc::now()),
    };

    let job = heatmap_generation_job::Entity::insert(job)
        .exec_with_returning(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to create job: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create generation job")
        })?;

    Ok(HttpResponse::Accepted().json(GenerationJobResponse::from(job)))
}

/// POST /api/heatmap/generate/:theme_slug
/// Manually trigger heatmap generation for a specific theme
pub async fn trigger_theme_generation(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;
    let theme_slug = path.into_inner();

    // Find the theme
    let theme = crate::models::heatmap_theme::Entity::find()
        .filter(crate::models::heatmap_theme::Column::UserId.eq(user_id))
        .filter(crate::models::heatmap_theme::Column::Slug.eq(&theme_slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let theme = match theme {
        Some(t) => t,
        None => return Err(actix_web::error::ErrorNotFound("Theme not found")),
    };

    // Create a generation job for this specific theme
    let job = heatmap_generation_job::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        theme_id: Set(Some(theme.id)),
        status: Set(heatmap_generation_job::GenerationJobStatus::Pending),
        scheduled_at: Set(chrono::Utc::now()),
        started_at: Set(None),
        completed_at: Set(None),
        error_message: Set(None),
        retry_count: Set(0),
        max_retries: Set(3),
        is_manual: Set(true),
        priority: Set(10), // Higher priority for manual triggers
        created_at: Set(chrono::Utc::now()),
    };

    let job = heatmap_generation_job::Entity::insert(job)
        .exec_with_returning(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to create job: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create generation job")
        })?;

    Ok(HttpResponse::Accepted().json(GenerationJobResponse::from(job)))
}

/// GET /api/heatmap/generated
/// List all generated heatmaps for the user
pub async fn list_generated_heatmaps(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let heatmaps = generated_heatmap::Entity::find()
        .filter(generated_heatmap::Column::UserId.eq(user_id))
        .order_by_desc(generated_heatmap::Column::GeneratedAt)
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let response: Vec<GeneratedHeatmapInfo> = heatmaps.into_iter()
        .map(GeneratedHeatmapInfo::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/heatmap/jobs
/// List generation jobs for the user
pub async fn list_generation_jobs(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    let limit: u64 = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let mut query_builder = heatmap_generation_job::Entity::find()
        .filter(heatmap_generation_job::Column::UserId.eq(user_id))
        .order_by_desc(heatmap_generation_job::Column::CreatedAt);

    // Filter by status if provided
    if let Some(status_str) = query.get("status") {
        let status = match status_str.as_str() {
            "pending" => heatmap_generation_job::GenerationJobStatus::Pending,
            "processing" => heatmap_generation_job::GenerationJobStatus::Processing,
            "completed" => heatmap_generation_job::GenerationJobStatus::Completed,
            "failed" => heatmap_generation_job::GenerationJobStatus::Failed,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
        };
        query_builder = query_builder.filter(heatmap_generation_job::Column::Status.eq(status));
    }

    let jobs = query_builder
        .limit(limit)
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let response: Vec<GenerationJobResponse> = jobs.into_iter()
        .map(GenerationJobResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/heatmap/preview
/// Generate a preview SVG for theme configuration using real user data
pub async fn preview_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<PreviewThemeRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e))
    })?;

    // Debug: log the incoming request
    log::info!("Preview request - color_scheme: {}, background_color: {:?}, custom_colors: {:?}",
        payload.color_scheme,
        payload.background_color,
        payload.custom_colors
    );

    // Create a temporary theme model from the request
    let theme = heatmap_theme::Model {
        id: Uuid::new_v4(),
        user_id,
        name: "Preview".to_string(),
        slug: "preview".to_string(),
        description: None,
        is_default: false,
        theme_mode: parse_theme_mode(&payload.theme_mode)?,
        color_scheme: parse_color_scheme(&payload.color_scheme)?,
        custom_colors: payload.custom_colors.clone().map(|colors| {
            serde_json::to_value(colors).unwrap_or(serde_json::Value::Null)
        }),
        background_color: payload.background_color.clone().unwrap_or("#ffffff".to_string()),
        border_color: payload.border_color.clone().unwrap_or("#d1d5da".to_string()),
        text_color: payload.text_color.clone().unwrap_or("#24292e".to_string()),
        empty_cell_color: payload.empty_cell_color.clone().unwrap_or("#ebedf0".to_string()),
        cell_size: payload.cell_size.unwrap_or(10),
        cell_gap: payload.cell_gap.unwrap_or(2),
        cell_border_radius: payload.cell_border_radius.unwrap_or(2),
        cell_border_width: payload.cell_border_width.unwrap_or(0),
        cell_border_color: payload.cell_border_color.clone().unwrap_or("#d1d5da".to_string()),
        heatmap_width: payload.heatmap_width,
        heatmap_height: payload.heatmap_height,
        padding_top: payload.padding_top.unwrap_or(20),
        padding_right: payload.padding_right.unwrap_or(20),
        padding_bottom: payload.padding_bottom.unwrap_or(17),
        padding_left: payload.padding_left.unwrap_or(20),
        day_label_width: payload.day_label_width.unwrap_or(28),
        month_label_height: payload.month_label_height.unwrap_or(15),
        title_height: payload.title_height.unwrap_or(30),
        legend_height: payload.legend_height.unwrap_or(8),
        show_month_labels: payload.show_month_labels.unwrap_or(true),
        show_day_labels: payload.show_day_labels.unwrap_or(true),
        show_legend: payload.show_legend.unwrap_or(true),
        show_total_count: payload.show_total_count.unwrap_or(true),
        show_username: payload.show_username.unwrap_or(true),
        show_watermark: payload.show_watermark.unwrap_or(true),
        font_family: payload.font_family.clone().unwrap_or("sans-serif".to_string()),
        font_size: payload.font_size.unwrap_or(10),
        legend_position: payload.legend_position.clone().unwrap_or("bottom".to_string()),
        output_formats: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Fetch user to get username
    let user_model = user::Entity::find_by_id(user_id)
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    // Fetch user's generation settings (or use defaults)
    let settings = heatmap_generation_setting::Entity::find()
        .filter(heatmap_generation_setting::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let mut settings = settings.unwrap_or_else(|| {
        // Default settings
        heatmap_generation_setting::Model {
            id: Uuid::new_v4(),
            user_id,
            update_interval_minutes: 60,
            auto_generation_enabled: false,
            date_range_days: 365,
            include_private_contributions: true,
            storage_path: None,
            last_scheduled_generation_at: None,
            next_scheduled_generation_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    });

    // Override date range if preview dates are provided
    if let (Some(from_date), Some(to_date)) = (&payload.preview_from_date, &payload.preview_to_date) {
        use chrono::NaiveDate;

        // Parse dates
        let from = NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
            .map_err(|_| actix_web::error::ErrorBadRequest("Invalid from_date format"))?;
        let to = NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
            .map_err(|_| actix_web::error::ErrorBadRequest("Invalid to_date format"))?;

        // Calculate the number of days
        let days = (to - from).num_days() + 1;

        if days < 1 || days > 730 {
            return Err(actix_web::error::ErrorBadRequest(
                "Preview date range must be between 1 and 730 days"
            ));
        }

        settings.date_range_days = days as i32;
    }

    // Generate heatmap using real user contribution data
    let generator = HeatmapGenerator::new(db.get_ref().clone());

    // Fetch real contribution data
    let heatmap_data = generator.fetch_contribution_data(user_id, &settings)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch contribution data: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch contribution data")
        })?;

    // Determine the format for preview (default to svg)
    let format_str = payload.preview_format.as_deref().unwrap_or("svg");
    let format = match format_str {
        "svg" => heatmap_theme::HeatmapFormat::Svg,
        "png" => heatmap_theme::HeatmapFormat::Png,
        "jpeg" => heatmap_theme::HeatmapFormat::Jpeg,
        "webp" => heatmap_theme::HeatmapFormat::WebP,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid preview format")),
    };

    // Generate in the requested format with username
    let content = generator.generate_heatmap_with_username(&theme, &heatmap_data, &format, Some(&user_model.username))
        .map_err(|e| {
            log::error!("Failed to generate preview: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to generate preview")
        })?;

    // Set appropriate content type
    let content_type = match format {
        heatmap_theme::HeatmapFormat::Svg => "image/svg+xml",
        heatmap_theme::HeatmapFormat::Png => "image/png",
        heatmap_theme::HeatmapFormat::Jpeg => "image/jpeg",
        heatmap_theme::HeatmapFormat::WebP => "image/webp",
    };

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .body(content))
}

#[derive(Debug, Deserialize)]
pub struct PreviewThemeRequest {
    pub theme_mode: String,
    pub color_scheme: String,
    pub custom_colors: Option<Vec<String>>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub text_color: Option<String>,
    pub empty_cell_color: Option<String>,
    pub cell_size: Option<i32>,
    pub cell_gap: Option<i32>,
    pub cell_border_radius: Option<i32>,
    pub cell_border_width: Option<i32>,
    pub cell_border_color: Option<String>,
    pub heatmap_width: Option<i32>,
    pub heatmap_height: Option<i32>,
    pub padding_top: Option<i32>,
    pub padding_right: Option<i32>,
    pub padding_bottom: Option<i32>,
    pub padding_left: Option<i32>,
    pub day_label_width: Option<i32>,
    pub month_label_height: Option<i32>,
    pub title_height: Option<i32>,
    pub legend_height: Option<i32>,
    pub show_month_labels: Option<bool>,
    pub show_day_labels: Option<bool>,
    pub show_legend: Option<bool>,
    pub show_total_count: Option<bool>,
    pub show_username: Option<bool>,
    pub show_watermark: Option<bool>,
    pub font_family: Option<String>,
    pub font_size: Option<i32>,
    pub legend_position: Option<String>,
    pub preview_from_date: Option<String>,
    pub preview_to_date: Option<String>,
    pub preview_format: Option<String>, // svg, png, jpeg, webp
}

fn parse_theme_mode(mode: &str) -> Result<heatmap_theme::ThemeMode, actix_web::Error> {
    match mode {
        "light" => Ok(heatmap_theme::ThemeMode::Light),
        "dark" => Ok(heatmap_theme::ThemeMode::Dark),
        _ => Err(actix_web::error::ErrorBadRequest("Invalid theme mode")),
    }
}

fn parse_color_scheme(scheme: &str) -> Result<heatmap_theme::HeatmapColorScheme, actix_web::Error> {
    match scheme {
        "github_green" => Ok(heatmap_theme::HeatmapColorScheme::GitHubGreen),
        "github_blue" => Ok(heatmap_theme::HeatmapColorScheme::GitHubBlue),
        "halloween" => Ok(heatmap_theme::HeatmapColorScheme::Halloween),
        "winter" => Ok(heatmap_theme::HeatmapColorScheme::Winter),
        "ocean" => Ok(heatmap_theme::HeatmapColorScheme::Ocean),
        "sunset" => Ok(heatmap_theme::HeatmapColorScheme::Sunset),
        "forest" => Ok(heatmap_theme::HeatmapColorScheme::Forest),
        "monochrome" => Ok(heatmap_theme::HeatmapColorScheme::Monochrome),
        "rainbow" => Ok(heatmap_theme::HeatmapColorScheme::Rainbow),
        "custom" => Ok(heatmap_theme::HeatmapColorScheme::Custom),
        _ => Err(actix_web::error::ErrorBadRequest("Invalid color scheme")),
    }
}
