use actix_web::{web, HttpResponse, Responder};
use sea_orm::sea_query::Expr;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{generated_heatmap, heatmap_theme};

// ============ Request/Response DTOs ============

#[derive(Debug, Serialize)]
pub struct HeatmapThemeResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub theme_mode: String,
    pub color_scheme: String,
    pub custom_colors: Option<serde_json::Value>,
    pub background_color: String,
    pub border_color: String,
    pub text_color: String,
    pub empty_cell_color: String,
    pub cell_size: i32,
    pub cell_gap: i32,
    pub cell_border_radius: i32,
    pub cell_border_width: i32,
    pub cell_border_color: String,
    pub heatmap_width: Option<i32>,
    pub heatmap_height: Option<i32>,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub day_label_width: i32,
    pub month_label_height: i32,
    pub title_height: i32,
    pub legend_height: i32,
    pub show_month_labels: bool,
    pub show_day_labels: bool,
    pub show_legend: bool,
    pub show_total_count: bool,
    pub show_username: bool,
    pub show_watermark: bool,
    pub font_family: String,
    pub font_size: i32,
    pub legend_position: String,
    pub output_formats: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<heatmap_theme::Model> for HeatmapThemeResponse {
    fn from(model: heatmap_theme::Model) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            slug: model.slug,
            description: model.description,
            is_default: model.is_default,
            theme_mode: match model.theme_mode {
                heatmap_theme::ThemeMode::Light => "light".to_string(),
                heatmap_theme::ThemeMode::Dark => "dark".to_string(),
            },
            color_scheme: match model.color_scheme {
                heatmap_theme::HeatmapColorScheme::GitHubGreen => "github_green".to_string(),
                heatmap_theme::HeatmapColorScheme::GitHubBlue => "github_blue".to_string(),
                heatmap_theme::HeatmapColorScheme::Halloween => "halloween".to_string(),
                heatmap_theme::HeatmapColorScheme::Winter => "winter".to_string(),
                heatmap_theme::HeatmapColorScheme::Ocean => "ocean".to_string(),
                heatmap_theme::HeatmapColorScheme::Sunset => "sunset".to_string(),
                heatmap_theme::HeatmapColorScheme::Forest => "forest".to_string(),
                heatmap_theme::HeatmapColorScheme::Monochrome => "monochrome".to_string(),
                heatmap_theme::HeatmapColorScheme::Rainbow => "rainbow".to_string(),
                heatmap_theme::HeatmapColorScheme::Custom => "custom".to_string(),
            },
            custom_colors: model.custom_colors,
            background_color: model.background_color,
            border_color: model.border_color,
            text_color: model.text_color,
            empty_cell_color: model.empty_cell_color,
            cell_size: model.cell_size,
            cell_gap: model.cell_gap,
            cell_border_radius: model.cell_border_radius,
            cell_border_width: model.cell_border_width,
            cell_border_color: model.cell_border_color,
            heatmap_width: model.heatmap_width,
            heatmap_height: model.heatmap_height,
            padding_top: model.padding_top,
            padding_right: model.padding_right,
            padding_bottom: model.padding_bottom,
            padding_left: model.padding_left,
            day_label_width: model.day_label_width,
            month_label_height: model.month_label_height,
            title_height: model.title_height,
            legend_height: model.legend_height,
            show_month_labels: model.show_month_labels,
            show_day_labels: model.show_day_labels,
            show_legend: model.show_legend,
            show_total_count: model.show_total_count,
            show_username: model.show_username,
            show_watermark: model.show_watermark,
            font_family: model.font_family,
            font_size: model.font_size,
            legend_position: model.legend_position,
            output_formats: model
                .output_formats
                .iter()
                .map(|f| format_to_string_out(f))
                .collect(),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

fn format_to_string_out(format: &heatmap_theme::HeatmapFormat) -> String {
    match format {
        heatmap_theme::HeatmapFormat::Svg => "svg".to_string(),
        heatmap_theme::HeatmapFormat::Png => "png".to_string(),
        heatmap_theme::HeatmapFormat::Jpeg => "jpeg".to_string(),
        heatmap_theme::HeatmapFormat::WebP => "webp".to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateThemeRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
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
    pub output_formats: Option<Vec<String>>,
}

fn parse_output_formats(
    formats: &[String],
) -> Result<Vec<heatmap_theme::HeatmapFormat>, actix_web::Error> {
    let mut parsed = Vec::new();
    for format in formats {
        let f = match format.as_str() {
            "svg" => heatmap_theme::HeatmapFormat::Svg,
            "png" => heatmap_theme::HeatmapFormat::Png,
            "jpeg" => heatmap_theme::HeatmapFormat::Jpeg,
            "webp" => heatmap_theme::HeatmapFormat::WebP,
            _ => {
                return Err(actix_web::error::ErrorBadRequest(format!(
                    "Invalid format: {}",
                    format
                )))
            }
        };
        parsed.push(f);
    }
    Ok(parsed)
}

#[derive(Debug, Deserialize)]
pub struct UpdateThemeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub theme_mode: Option<String>,
    pub color_scheme: Option<String>,
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
    pub output_formats: Option<Vec<String>>,
}

// ============ Theme Handlers ============

/// GET /api/heatmap/themes
/// List all themes for the authenticated user
pub async fn list_themes(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    let themes = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .order_by_asc(heatmap_theme::Column::CreatedAt)
        .all(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let response: Vec<HeatmapThemeResponse> =
        themes.into_iter().map(HeatmapThemeResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/heatmap/themes/:slug
/// Get a specific theme by slug
pub async fn get_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;
    let slug = path.into_inner();

    let theme = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    match theme {
        Some(theme) => Ok(HttpResponse::Ok().json(HeatmapThemeResponse::from(theme))),
        None => Err(actix_web::error::ErrorNotFound("Theme not found")),
    }
}

/// POST /api/heatmap/themes
/// Create a new theme
pub async fn create_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    payload: web::Json<CreateThemeRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;

    // Check if slug already exists for this user
    let existing = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&payload.slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if existing.is_some() {
        return Err(actix_web::error::ErrorConflict(
            "Theme with this slug already exists",
        ));
    }

    // Parse theme mode
    let theme_mode = match payload.theme_mode.as_str() {
        "light" => heatmap_theme::ThemeMode::Light,
        "dark" => heatmap_theme::ThemeMode::Dark,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid theme mode")),
    };

    // Parse color scheme
    let color_scheme = match payload.color_scheme.as_str() {
        "github_green" => heatmap_theme::HeatmapColorScheme::GitHubGreen,
        "github_blue" => heatmap_theme::HeatmapColorScheme::GitHubBlue,
        "halloween" => heatmap_theme::HeatmapColorScheme::Halloween,
        "winter" => heatmap_theme::HeatmapColorScheme::Winter,
        "ocean" => heatmap_theme::HeatmapColorScheme::Ocean,
        "sunset" => heatmap_theme::HeatmapColorScheme::Sunset,
        "forest" => heatmap_theme::HeatmapColorScheme::Forest,
        "monochrome" => heatmap_theme::HeatmapColorScheme::Monochrome,
        "rainbow" => heatmap_theme::HeatmapColorScheme::Rainbow,
        "custom" => heatmap_theme::HeatmapColorScheme::Custom,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid color scheme")),
    };

    let custom_colors = payload
        .custom_colors
        .as_ref()
        .map(|colors| serde_json::to_value(colors).unwrap());

    // Parse output formats
    let output_formats = if let Some(ref formats) = payload.output_formats {
        parse_output_formats(formats)?
    } else {
        vec![heatmap_theme::HeatmapFormat::Png]
    };

    // Set default colors based on theme mode
    let (default_bg, default_text, default_empty) = match theme_mode {
        heatmap_theme::ThemeMode::Light => ("#ffffff", "#24292e", "#ebedf0"),
        heatmap_theme::ThemeMode::Dark => ("#0d1117", "#c9d1d9", "#161b22"),
    };

    let new_theme = heatmap_theme::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(payload.name.clone()),
        slug: Set(payload.slug.clone()),
        description: Set(payload.description.clone()),
        is_default: Set(false),
        theme_mode: Set(theme_mode),
        color_scheme: Set(color_scheme),
        custom_colors: Set(custom_colors),
        background_color: Set(payload
            .background_color
            .clone()
            .unwrap_or_else(|| default_bg.to_string())),
        border_color: Set(payload
            .border_color
            .clone()
            .unwrap_or_else(|| "#d1d5da".to_string())),
        text_color: Set(payload
            .text_color
            .clone()
            .unwrap_or_else(|| default_text.to_string())),
        empty_cell_color: Set(payload
            .empty_cell_color
            .clone()
            .unwrap_or_else(|| default_empty.to_string())),
        cell_size: Set(payload.cell_size.unwrap_or(10)),
        cell_gap: Set(payload.cell_gap.unwrap_or(2)),
        cell_border_radius: Set(payload.cell_border_radius.unwrap_or(2)),
        cell_border_width: Set(payload.cell_border_width.unwrap_or(0)),
        cell_border_color: Set(payload
            .cell_border_color
            .clone()
            .unwrap_or_else(|| "#d1d5da".to_string())),
        heatmap_width: Set(payload.heatmap_width),
        heatmap_height: Set(payload.heatmap_height),
        padding_top: Set(payload.padding_top.unwrap_or(20)),
        padding_right: Set(payload.padding_right.unwrap_or(20)),
        padding_bottom: Set(payload.padding_bottom.unwrap_or(17)),
        padding_left: Set(payload.padding_left.unwrap_or(20)),
        day_label_width: Set(payload.day_label_width.unwrap_or(28)),
        month_label_height: Set(payload.month_label_height.unwrap_or(15)),
        title_height: Set(payload.title_height.unwrap_or(30)),
        legend_height: Set(payload.legend_height.unwrap_or(8)),
        show_month_labels: Set(payload.show_month_labels.unwrap_or(true)),
        show_day_labels: Set(payload.show_day_labels.unwrap_or(true)),
        show_legend: Set(payload.show_legend.unwrap_or(true)),
        show_total_count: Set(payload.show_total_count.unwrap_or(true)),
        show_username: Set(payload.show_username.unwrap_or(true)),
        show_watermark: Set(payload.show_watermark.unwrap_or(true)),
        font_family: Set(payload
            .font_family
            .clone()
            .unwrap_or_else(|| "Nimbus Sans".to_string())),
        font_size: Set(payload.font_size.unwrap_or(10)),
        legend_position: Set(payload
            .legend_position
            .clone()
            .unwrap_or_else(|| "bottom".to_string())),
        output_formats: Set(output_formats),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let theme = heatmap_theme::Entity::insert(new_theme)
        .exec_with_returning(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to create theme: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create theme")
        })?;

    Ok(HttpResponse::Created().json(HeatmapThemeResponse::from(theme)))
}

/// PUT /api/heatmap/themes/:slug
/// Update an existing theme
pub async fn update_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    payload: web::Json<UpdateThemeRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;
    let slug = path.into_inner();

    let theme = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&slug))
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

    let mut active_theme: heatmap_theme::ActiveModel = theme.into();

    if let Some(name) = &payload.name {
        active_theme.name = Set(name.clone());
    }

    if let Some(description) = &payload.description {
        active_theme.description = Set(Some(description.clone()));
    }

    if let Some(theme_mode_str) = &payload.theme_mode {
        let theme_mode = match theme_mode_str.as_str() {
            "light" => heatmap_theme::ThemeMode::Light,
            "dark" => heatmap_theme::ThemeMode::Dark,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid theme mode")),
        };
        active_theme.theme_mode = Set(theme_mode);
    }

    if let Some(color_scheme_str) = &payload.color_scheme {
        let color_scheme = match color_scheme_str.as_str() {
            "github_green" => heatmap_theme::HeatmapColorScheme::GitHubGreen,
            "github_blue" => heatmap_theme::HeatmapColorScheme::GitHubBlue,
            "halloween" => heatmap_theme::HeatmapColorScheme::Halloween,
            "winter" => heatmap_theme::HeatmapColorScheme::Winter,
            "ocean" => heatmap_theme::HeatmapColorScheme::Ocean,
            "sunset" => heatmap_theme::HeatmapColorScheme::Sunset,
            "forest" => heatmap_theme::HeatmapColorScheme::Forest,
            "monochrome" => heatmap_theme::HeatmapColorScheme::Monochrome,
            "rainbow" => heatmap_theme::HeatmapColorScheme::Rainbow,
            "custom" => heatmap_theme::HeatmapColorScheme::Custom,
            _ => return Err(actix_web::error::ErrorBadRequest("Invalid color scheme")),
        };
        active_theme.color_scheme = Set(color_scheme);
    }

    if let Some(custom_colors) = &payload.custom_colors {
        let colors_json = serde_json::to_value(custom_colors).unwrap();
        active_theme.custom_colors = Set(Some(colors_json));
    }

    // Update all other optional fields
    macro_rules! update_field {
        ($field:ident) => {
            if let Some(value) = &payload.$field {
                active_theme.$field = Set(value.clone());
            }
        };
        ($field:ident, opt) => {
            if let Some(value) = payload.$field {
                active_theme.$field = Set(Some(value));
            }
        };
        ($field:ident, val) => {
            if let Some(value) = payload.$field {
                active_theme.$field = Set(value);
            }
        };
        ($field:ident, bool) => {
            if let Some(value) = payload.$field {
                active_theme.$field = Set(value);
            }
        };
    }

    update_field!(background_color);
    update_field!(border_color);
    update_field!(text_color);
    update_field!(empty_cell_color);
    update_field!(cell_border_color);
    update_field!(font_family);
    update_field!(legend_position);
    update_field!(cell_size, val);
    update_field!(cell_gap, val);
    update_field!(cell_border_radius, val);
    update_field!(cell_border_width, val);
    update_field!(heatmap_width, opt);
    update_field!(heatmap_height, opt);
    update_field!(padding_top, val);
    update_field!(padding_right, val);
    update_field!(padding_bottom, val);
    update_field!(padding_left, val);
    update_field!(day_label_width, val);
    update_field!(month_label_height, val);
    update_field!(title_height, val);
    update_field!(legend_height, val);
    update_field!(font_size, val);
    update_field!(show_month_labels, bool);
    update_field!(show_day_labels, bool);
    update_field!(show_legend, bool);
    update_field!(show_total_count, bool);
    update_field!(show_username, bool);
    update_field!(show_watermark, bool);

    if let Some(ref formats) = payload.output_formats {
        let parsed_formats = parse_output_formats(formats)?;
        active_theme.output_formats = Set(parsed_formats);
    }

    active_theme.updated_at = Set(chrono::Utc::now());

    let updated_theme = active_theme.update(db.as_ref()).await.map_err(|e| {
        log::error!("Failed to update theme: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update theme")
    })?;

    // Invalidate generated heatmaps for this theme
    let _ = generated_heatmap::Entity::update_many()
        .filter(generated_heatmap::Column::ThemeId.eq(updated_theme.id))
        .col_expr(generated_heatmap::Column::IsValid, Expr::value(false))
        .exec(db.as_ref())
        .await;

    Ok(HttpResponse::Ok().json(HeatmapThemeResponse::from(updated_theme)))
}

/// DELETE /api/heatmap/themes/:slug
/// Delete a theme
pub async fn delete_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;
    let slug = path.into_inner();

    let theme = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&slug))
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

    // Don't allow deleting default theme
    if theme.is_default {
        return Err(actix_web::error::ErrorBadRequest(
            "Cannot delete default theme",
        ));
    }

    heatmap_theme::Entity::delete_by_id(theme.id)
        .exec(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to delete theme: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to delete theme")
        })?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/heatmap/themes/:slug/set-default
/// Set a theme as the default
pub async fn set_default_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;
    let slug = path.into_inner();

    // Start a transaction
    let txn = db.begin().await.map_err(|e| {
        log::error!("Failed to start transaction: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Unset all defaults for this user
    heatmap_theme::Entity::update_many()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .col_expr(heatmap_theme::Column::IsDefault, Expr::value(false))
        .exec(&txn)
        .await
        .map_err(|e| {
            log::error!("Failed to unset defaults: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Set the new default
    let theme = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&slug))
        .one(&txn)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let theme = match theme {
        Some(t) => t,
        None => {
            let _ = txn.rollback().await;
            return Err(actix_web::error::ErrorNotFound("Theme not found"));
        }
    };

    let mut active_theme: heatmap_theme::ActiveModel = theme.into();
    active_theme.is_default = Set(true);
    active_theme.updated_at = Set(chrono::Utc::now());

    let updated_theme = active_theme.update(&txn).await.map_err(|e| {
        log::error!("Failed to set default: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to set default")
    })?;

    txn.commit().await.map_err(|e| {
        log::error!("Failed to commit transaction: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(HeatmapThemeResponse::from(updated_theme)))
}

#[derive(Debug, Deserialize)]
pub struct DuplicateThemeRequest {
    pub new_name: String,
    pub new_slug: String,
}

/// POST /api/heatmap/themes/:slug/duplicate
/// Duplicate an existing theme
pub async fn duplicate_theme(
    db: web::Data<DatabaseConnection>,
    user_claims: web::ReqData<crate::middleware::auth::Claims>,
    path: web::Path<String>,
    payload: web::Json<DuplicateThemeRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = Uuid::parse_str(&user_claims.sub)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid user ID: {}", e)))?;
    let source_slug = path.into_inner();

    // Find the source theme
    let source_theme = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&source_slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let source_theme = match source_theme {
        Some(t) => t,
        None => return Err(actix_web::error::ErrorNotFound("Source theme not found")),
    };

    // Check if new slug already exists
    let existing = heatmap_theme::Entity::find()
        .filter(heatmap_theme::Column::UserId.eq(user_id))
        .filter(heatmap_theme::Column::Slug.eq(&payload.new_slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if existing.is_some() {
        return Err(actix_web::error::ErrorConflict(
            "Theme with this slug already exists",
        ));
    }

    // Create duplicate with all settings from source theme
    let new_theme = heatmap_theme::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(payload.new_name.clone()),
        slug: Set(payload.new_slug.clone()),
        description: Set(source_theme.description.clone()),
        is_default: Set(false), // Duplicates are never default
        theme_mode: Set(source_theme.theme_mode.clone()),
        color_scheme: Set(source_theme.color_scheme.clone()),
        custom_colors: Set(source_theme.custom_colors.clone()),
        background_color: Set(source_theme.background_color.clone()),
        border_color: Set(source_theme.border_color.clone()),
        text_color: Set(source_theme.text_color.clone()),
        empty_cell_color: Set(source_theme.empty_cell_color.clone()),
        cell_size: Set(source_theme.cell_size),
        cell_gap: Set(source_theme.cell_gap),
        cell_border_radius: Set(source_theme.cell_border_radius),
        cell_border_width: Set(source_theme.cell_border_width),
        cell_border_color: Set(source_theme.cell_border_color.clone()),
        heatmap_width: Set(source_theme.heatmap_width),
        heatmap_height: Set(source_theme.heatmap_height),
        padding_top: Set(source_theme.padding_top),
        padding_right: Set(source_theme.padding_right),
        padding_bottom: Set(source_theme.padding_bottom),
        padding_left: Set(source_theme.padding_left),
        day_label_width: Set(source_theme.day_label_width),
        month_label_height: Set(source_theme.month_label_height),
        title_height: Set(source_theme.title_height),
        legend_height: Set(source_theme.legend_height),
        show_month_labels: Set(source_theme.show_month_labels),
        show_day_labels: Set(source_theme.show_day_labels),
        show_legend: Set(source_theme.show_legend),
        show_total_count: Set(source_theme.show_total_count),
        show_username: Set(source_theme.show_username),
        show_watermark: Set(source_theme.show_watermark),
        font_family: Set(source_theme.font_family.clone()),
        font_size: Set(source_theme.font_size),
        legend_position: Set(source_theme.legend_position.clone()),
        output_formats: Set(source_theme.output_formats.clone()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let duplicated_theme = heatmap_theme::Entity::insert(new_theme)
        .exec_with_returning(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to duplicate theme: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to duplicate theme")
        })?;

    log::info!(
        "Theme '{}' duplicated to '{}' for user {}",
        source_slug,
        payload.new_slug,
        user_id
    );

    Ok(HttpResponse::Created().json(HeatmapThemeResponse::from(duplicated_theme)))
}
