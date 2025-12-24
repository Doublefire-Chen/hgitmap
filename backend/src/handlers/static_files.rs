use actix_web::{web, Responder};
use actix_files::NamedFile;
use sea_orm::*;
use sea_orm::sea_query::Expr;
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::{generated_heatmap, heatmap_generation_setting, user};
use crate::services::heatmap_generator::HeatmapGenerator;

/// GET /static/heatmaps/:user_id/:filename
/// Serve generated heatmap files
pub async fn serve_heatmap(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, actix_web::Error> {
    let (user_id_str, filename) = path.into_inner();

    // Construct file path (using default directory)
    let base_dir = "static/heatmaps";

    let file_path = PathBuf::from(&base_dir)
        .join(&user_id_str)
        .join(&filename);

    // Check if file exists
    if !file_path.exists() {
        return Err(actix_web::error::ErrorNotFound("Heatmap not found"));
    }

    // Update access count in database (optional, async)
    let db_clone = db.clone();
    let file_path_str = file_path.to_string_lossy().to_string();

    tokio::spawn(async move {
        let _ = update_access_count(&db_clone, &file_path_str).await;
    });

    // Serve the file
    let named_file = NamedFile::open(file_path).map_err(|e| {
        log::error!("Failed to open file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to serve file")
    })?;

    Ok(named_file)
}

/// GET /embed/:username/:theme_slug.{format}
/// Public endpoint for embedding heatmaps (e.g., in GitHub README)
/// Generates the heatmap on-demand if it doesn't exist yet
pub async fn serve_embed(
    db: web::Data<DatabaseConnection>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, actix_web::Error> {
    let (username, theme_file) = path.into_inner();

    // Parse theme slug and format from filename
    let parts: Vec<&str> = theme_file.rsplitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(actix_web::error::ErrorBadRequest("Invalid filename format"));
    }

    let format_str = parts[0];
    let theme_slug = parts[1];

    // Find user by username
    let user = crate::models::user::Entity::find()
        .filter(crate::models::user::Column::Username.eq(&username))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    // Find theme
    let theme = crate::models::heatmap_theme::Entity::find()
        .filter(crate::models::heatmap_theme::Column::UserId.eq(user.id))
        .filter(crate::models::heatmap_theme::Column::Slug.eq(theme_slug))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Theme not found"))?;

    // Parse format
    let format = match format_str {
        "svg" => crate::models::heatmap_theme::HeatmapFormat::Svg,
        "png" => crate::models::heatmap_theme::HeatmapFormat::Png,
        "jpg" | "jpeg" => crate::models::heatmap_theme::HeatmapFormat::Jpeg,
        "webp" => crate::models::heatmap_theme::HeatmapFormat::WebP,
        _ => return Err(actix_web::error::ErrorBadRequest("Unsupported format")),
    };

    // Try to find existing valid generated heatmap
    let existing_generated = generated_heatmap::Entity::find()
        .filter(generated_heatmap::Column::UserId.eq(user.id))
        .filter(generated_heatmap::Column::ThemeId.eq(theme.id))
        .filter(generated_heatmap::Column::Format.eq(format.clone()))
        .filter(generated_heatmap::Column::IsValid.eq(true))
        .one(db.as_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Check if we need to generate the heatmap
    let file_path = if let Some(generated) = existing_generated {
        let path = PathBuf::from(&generated.file_path);

        // If file exists, use it
        if path.exists() {
            // Update access count
            let db_clone = db.clone();
            let generated_id = generated.id;
            tokio::spawn(async move {
                let _ = increment_access_count(&db_clone, generated_id).await;
            });

            path
        } else {
            // File missing, regenerate
            log::warn!("Heatmap file missing for user {}, theme {}, regenerating", user.id, theme.id);
            generate_heatmap_on_demand(db.as_ref(), &user.id, &theme, &format).await?
        }
    } else {
        // No generated heatmap found, generate on-demand
        log::info!("Heatmap not found for user {}, theme {} ({}), generating on-demand", user.id, theme.slug, format_str);
        generate_heatmap_on_demand(db.as_ref(), &user.id, &theme, &format).await?
    };

    // Serve the file
    let named_file = NamedFile::open(file_path).map_err(|e| {
        log::error!("Failed to open file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to serve file")
    })?;

    Ok(named_file)
}

/// Generate a heatmap on-demand and save it to disk and database
async fn generate_heatmap_on_demand(
    db: &DatabaseConnection,
    user_id: &Uuid,
    theme: &crate::models::heatmap_theme::Model,
    format: &crate::models::heatmap_theme::HeatmapFormat,
) -> Result<PathBuf, actix_web::Error> {
    use chrono::Utc;
    use std::fs;
    use std::io::Write;

    // Get user's generation settings
    let settings = heatmap_generation_setting::Entity::find()
        .filter(heatmap_generation_setting::Column::UserId.eq(*user_id))
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .unwrap_or_else(|| {
            // Default settings
            heatmap_generation_setting::Model {
                id: Uuid::new_v4(),
                user_id: *user_id,
                update_interval_minutes: 60,
                auto_generation_enabled: true,
                date_range_days: 365,
                include_private_contributions: true,
                storage_path: None,
                last_scheduled_generation_at: None,
                next_scheduled_generation_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        });

    // Create generator and fetch data
    let generator = HeatmapGenerator::new(db.clone());

    let start_time = std::time::Instant::now();

    // Fetch user to get username
    let user_model = user::Entity::find_by_id(*user_id)
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .ok_or_else(|| {
            log::error!("User not found: {}", user_id);
            actix_web::error::ErrorNotFound("User not found")
        })?;

    // Fetch contribution data
    let heatmap_data = generator.fetch_contribution_data(*user_id, &settings)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch contribution data: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch contribution data")
        })?;

    // Generate the heatmap in the requested format with username
    let content = generator.generate_heatmap_with_username(theme, &heatmap_data, format, Some(&user_model.username))
        .map_err(|e| {
            log::error!("Failed to generate heatmap: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to generate heatmap")
        })?;

    // Determine storage path
    let base_dir = settings.storage_path.as_deref().unwrap_or("static/heatmaps");
    let user_dir = PathBuf::from(base_dir).join(user_id.to_string());

    // Create directory if it doesn't exist
    fs::create_dir_all(&user_dir).map_err(|e| {
        log::error!("Failed to create directory: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create storage directory")
    })?;

    // Generate filename
    let extension = match format {
        crate::models::heatmap_theme::HeatmapFormat::Svg => "svg",
        crate::models::heatmap_theme::HeatmapFormat::Png => "png",
        crate::models::heatmap_theme::HeatmapFormat::Jpeg => "jpeg",
        crate::models::heatmap_theme::HeatmapFormat::WebP => "webp",
    };
    let filename = format!("{}.{}", theme.slug, extension);
    let file_path = user_dir.join(&filename);

    // Write file to disk
    let mut file = fs::File::create(&file_path).map_err(|e| {
        log::error!("Failed to create file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create heatmap file")
    })?;

    file.write_all(&content).map_err(|e| {
        log::error!("Failed to write file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to write heatmap file")
    })?;

    let file_size = content.len() as i64;
    let generation_duration = start_time.elapsed().as_millis() as i32;

    // Save to database
    let now = Utc::now();
    let (date_start, date_end) = (heatmap_data.date_range_start, heatmap_data.date_range_end);

    // Check if entry already exists
    let existing = generated_heatmap::Entity::find()
        .filter(generated_heatmap::Column::UserId.eq(*user_id))
        .filter(generated_heatmap::Column::ThemeId.eq(theme.id))
        .filter(generated_heatmap::Column::Format.eq(format.clone()))
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(existing) = existing {
        // Update existing record
        let mut active: generated_heatmap::ActiveModel = existing.into();
        active.file_path = Set(file_path.to_string_lossy().to_string());
        active.file_size_bytes = Set(Some(file_size));
        active.file_hash = Set(None); // Hash calculation optional
        active.generated_at = Set(now);
        active.generation_duration_ms = Set(Some(generation_duration));
        active.contribution_count = Set(heatmap_data.total_count);
        active.date_range_start = Set(date_start);
        active.date_range_end = Set(date_end);
        active.is_valid = Set(true);
        active.access_count = Set(1); // Reset access count on regeneration
        active.last_accessed_at = Set(Some(now));

        active.update(db).await.map_err(|e| {
            log::error!("Failed to update generated_heatmap: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to save heatmap metadata")
        })?;
    } else {
        // Create new record
        let new_generated = generated_heatmap::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(*user_id),
            theme_id: Set(theme.id),
            format: Set(format.clone()),
            file_path: Set(file_path.to_string_lossy().to_string()),
            file_size_bytes: Set(Some(file_size)),
            file_hash: Set(None), // Hash calculation optional
            generated_at: Set(now),
            generation_duration_ms: Set(Some(generation_duration)),
            contribution_count: Set(heatmap_data.total_count),
            date_range_start: Set(date_start),
            date_range_end: Set(date_end),
            access_count: Set(1),
            last_accessed_at: Set(Some(now)),
            is_valid: Set(true),
        };

        generated_heatmap::Entity::insert(new_generated)
            .exec(db)
            .await
            .map_err(|e| {
                log::error!("Failed to insert generated_heatmap: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to save heatmap metadata")
            })?;
    }

    log::info!("Generated heatmap on-demand for user {}, theme {} ({}) in {}ms",
        user_id, theme.slug, extension, generation_duration);

    Ok(file_path)
}

/// Update access count for a generated heatmap
async fn update_access_count(db: &DatabaseConnection, file_path: &str) -> Result<(), DbErr> {
    let heatmap = generated_heatmap::Entity::find()
        .filter(generated_heatmap::Column::FilePath.eq(file_path))
        .one(db)
        .await?;

    if let Some(heatmap) = heatmap {
        let mut active: generated_heatmap::ActiveModel = heatmap.into();
        active.access_count = Set(active.access_count.unwrap() + 1);
        active.last_accessed_at = Set(Some(chrono::Utc::now()));
        active.update(db).await?;
    }

    Ok(())
}

/// Increment access count by ID
async fn increment_access_count(db: &DatabaseConnection, id: uuid::Uuid) -> Result<(), DbErr> {
    generated_heatmap::Entity::update_many()
        .filter(generated_heatmap::Column::Id.eq(id))
        .col_expr(
            generated_heatmap::Column::AccessCount,
            Expr::col(generated_heatmap::Column::AccessCount).add(1),
        )
        .col_expr(
            generated_heatmap::Column::LastAccessedAt,
            Expr::value(chrono::Utc::now()),
        )
        .exec(db)
        .await?;

    Ok(())
}
