use anyhow::{Context, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use image::{ImageBuffer, ImageEncoder, RgbaImage};
use sea_orm::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use usvg::{TreeParsing, TreeTextToPath};

use crate::models::{
    contribution, generated_heatmap, git_platform_account, heatmap_generation_setting,
    heatmap_theme::{self, HeatmapFormat},
    user,
};

// Color palette definitions for different schemes
pub struct ColorPalette {
    pub colors: Vec<String>, // 5 colors from low to high intensity
}

impl ColorPalette {
    pub fn from_scheme(scheme: &heatmap_theme::HeatmapColorScheme) -> Self {
        let colors = match scheme {
            heatmap_theme::HeatmapColorScheme::GitHubGreen => vec![
                "#ebedf0".to_string(),
                "#9be9a8".to_string(),
                "#40c463".to_string(),
                "#30a14e".to_string(),
                "#216e39".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::GitHubBlue => vec![
                "#ebedf0".to_string(),
                "#9be9ff".to_string(),
                "#40c4ff".to_string(),
                "#2196f3".to_string(),
                "#1565c0".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Halloween => vec![
                "#ebedf0".to_string(),
                "#ffee4a".to_string(),
                "#ffc501".to_string(),
                "#fe9600".to_string(),
                "#03001c".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Winter => vec![
                "#ebedf0".to_string(),
                "#b6e3f4".to_string(),
                "#66c2e0".to_string(),
                "#2e8ab8".to_string(),
                "#1a5490".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Ocean => vec![
                "#ebedf0".to_string(),
                "#aadaff".to_string(),
                "#5fb3d9".to_string(),
                "#2a7fad".to_string(),
                "#0a4d80".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Sunset => vec![
                "#ebedf0".to_string(),
                "#ffd89b".to_string(),
                "#ff9a56".to_string(),
                "#ff6b35".to_string(),
                "#d94400".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Forest => vec![
                "#ebedf0".to_string(),
                "#c8e6c9".to_string(),
                "#81c784".to_string(),
                "#43a047".to_string(),
                "#2e7d32".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Monochrome => vec![
                "#ebedf0".to_string(),
                "#b0b0b0".to_string(),
                "#808080".to_string(),
                "#505050".to_string(),
                "#202020".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Rainbow => vec![
                "#ebedf0".to_string(),
                "#ffeb3b".to_string(),
                "#4caf50".to_string(),
                "#2196f3".to_string(),
                "#9c27b0".to_string(),
            ],
            heatmap_theme::HeatmapColorScheme::Custom => vec![
                "#ebedf0".to_string(), // Default, should be overridden
                "#9be9a8".to_string(),
                "#40c463".to_string(),
                "#30a14e".to_string(),
                "#216e39".to_string(),
            ],
        };

        Self { colors }
    }

    pub fn get_color_for_count(&self, count: i32, max_count: i32) -> &str {
        if count == 0 {
            return &self.colors[0];
        }

        if max_count == 0 {
            return &self.colors[1];
        }

        let ratio = count as f32 / max_count as f32;
        let index = match ratio {
            r if r >= 0.75 => 4,
            r if r >= 0.50 => 3,
            r if r >= 0.25 => 2,
            _ => 1,
        };

        &self.colors[index]
    }
}

// Contribution data for a single day
#[derive(Clone, Debug)]
pub struct DayContribution {
    #[allow(dead_code)]
    pub date: NaiveDate,
    pub count: i32,
}

// Heatmap data organized by weeks
#[derive(Debug)]
pub struct HeatmapData {
    pub weeks: Vec<Vec<DayContribution>>,
    pub max_count: i32,
    pub total_count: i32,
    pub date_range_start: NaiveDate,
    pub date_range_end: NaiveDate,
}

pub struct HeatmapGenerator {
    db: DatabaseConnection,
}

impl HeatmapGenerator {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate heatmaps for a specific theme
    pub async fn generate_for_theme(
        &self,
        user_id: uuid::Uuid,
        theme: &heatmap_theme::Model,
    ) -> Result<Vec<generated_heatmap::Model>> {
        log::info!(
            "Generating heatmaps for theme: {} (user: {})",
            theme.slug,
            user_id
        );

        let start_time = std::time::Instant::now();

        // Fetch user to get username
        let user_model = user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .context("User not found")?;

        // Fetch generation settings
        let settings = heatmap_generation_setting::Entity::find()
            .filter(heatmap_generation_setting::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?
            .context("Generation settings not found")?;

        // Fetch contribution data
        let heatmap_data = self.fetch_contribution_data(user_id, &settings).await?;

        // Generate SVG
        let svg_content = self.generate_svg(theme, &heatmap_data, Some(&user_model.username))?;

        // Ensure output directory exists
        let output_dir = self.get_output_directory(user_id)?;
        fs::create_dir_all(&output_dir)?;

        let mut generated_files = Vec::new();

        // Generate each requested format
        for format in &theme.output_formats {
            let file_path = self.get_file_path(&output_dir, &theme.slug, format);
            let file_content = match format {
                HeatmapFormat::Svg => svg_content.as_bytes().to_vec(),
                HeatmapFormat::Png => self.svg_to_png(&svg_content, theme)?,
                HeatmapFormat::Jpeg => self.svg_to_jpeg(&svg_content, theme)?,
                HeatmapFormat::WebP => self.svg_to_webp(&svg_content, theme)?,
            };

            // Write file
            fs::write(&file_path, &file_content)?;

            // Calculate file hash
            let file_hash = self.calculate_hash(&file_content);

            // Create database record
            let generated = generated_heatmap::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                user_id: Set(user_id),
                theme_id: Set(theme.id),
                format: Set(format.clone()),
                file_path: Set(file_path.to_string_lossy().to_string()),
                file_size_bytes: Set(Some(file_content.len() as i64)),
                file_hash: Set(Some(file_hash)),
                generated_at: Set(chrono::Utc::now()),
                generation_duration_ms: Set(Some(start_time.elapsed().as_millis() as i32)),
                contribution_count: Set(heatmap_data.total_count),
                date_range_start: Set(heatmap_data.date_range_start),
                date_range_end: Set(heatmap_data.date_range_end),
                access_count: Set(0),
                last_accessed_at: Set(None),
                is_valid: Set(true),
            };

            // Delete old generated file if exists
            generated_heatmap::Entity::delete_many()
                .filter(generated_heatmap::Column::UserId.eq(user_id))
                .filter(generated_heatmap::Column::ThemeId.eq(theme.id))
                .filter(generated_heatmap::Column::Format.eq(format.clone()))
                .exec(&self.db)
                .await?;

            let generated_model = generated_heatmap::Entity::insert(generated)
                .exec_with_returning(&self.db)
                .await?;

            generated_files.push(generated_model);

            log::info!(
                "Generated {} heatmap: {} ({} bytes)",
                format_to_string(format),
                file_path.display(),
                file_content.len()
            );
        }

        log::info!(
            "Generated {} files for theme '{}' in {:?}",
            generated_files.len(),
            theme.slug,
            start_time.elapsed()
        );

        Ok(generated_files)
    }

    /// Fetch contribution data for the user
    pub async fn fetch_contribution_data(
        &self,
        user_id: uuid::Uuid,
        settings: &heatmap_generation_setting::Model,
    ) -> Result<HeatmapData> {
        let end_date = Utc::now().date_naive();

        // Calculate the 365-day window start (or whatever date_range_days is set to)
        let window_start = end_date - Duration::days(settings.date_range_days as i64 - 1);

        // Go back to the Sunday before the window start to complete the first week
        let weekday = window_start.weekday().num_days_from_sunday();
        let start_date = if weekday != 0 {
            window_start - Duration::days(weekday as i64)
        } else {
            window_start
        };

        // Get all platform accounts for user
        let accounts = git_platform_account::Entity::find()
            .filter(git_platform_account::Column::UserId.eq(user_id))
            .filter(git_platform_account::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        let account_ids: Vec<uuid::Uuid> = accounts.iter().map(|a| a.id).collect();

        // Fetch contributions from the Sunday start date
        let contributions = contribution::Entity::find()
            .filter(contribution::Column::GitPlatformAccountId.is_in(account_ids))
            .filter(contribution::Column::ContributionDate.gte(start_date))
            .filter(contribution::Column::ContributionDate.lte(end_date))
            .all(&self.db)
            .await?;

        // Aggregate by date
        let mut contribution_map: std::collections::HashMap<NaiveDate, i32> =
            std::collections::HashMap::new();

        for contrib in contributions {
            let count = contribution_map
                .entry(contrib.contribution_date)
                .or_insert(0);
            *count += contrib.count;
        }

        // Filter private contributions if needed
        let contribution_map = if !settings.include_private_contributions {
            contribution_map
                .into_iter()
                .filter(|(_, count)| *count > 0)
                .collect()
        } else {
            contribution_map
        };

        // Build week structure (starting from Sunday)
        let mut weeks: Vec<Vec<DayContribution>> = Vec::new();
        let mut current_week: Vec<DayContribution> = Vec::new();

        let mut current_date = start_date;
        let mut max_count = 0;
        let mut total_count = 0;

        while current_date <= end_date {
            let count = *contribution_map.get(&current_date).unwrap_or(&0);

            if count > max_count {
                max_count = count;
            }
            total_count += count;

            current_week.push(DayContribution {
                date: current_date,
                count,
            });

            if current_week.len() == 7 {
                weeks.push(current_week.clone());
                current_week.clear();
            }

            current_date += Duration::days(1);
        }

        // Add remaining days
        if !current_week.is_empty() {
            while current_week.len() < 7 {
                current_week.push(DayContribution {
                    date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                    count: -1,
                });
            }
            weeks.push(current_week);
        }

        Ok(HeatmapData {
            weeks,
            max_count,
            total_count,
            date_range_start: start_date,
            date_range_end: end_date,
        })
    }

    /// Generate heatmap with username in the requested format (for embed URLs)
    pub fn generate_heatmap_with_username(
        &self,
        theme: &heatmap_theme::Model,
        data: &HeatmapData,
        format: &HeatmapFormat,
        username: Option<&str>,
    ) -> Result<Vec<u8>> {
        // Generate SVG with username
        let svg_content = self.generate_svg(theme, data, username)?;

        // Convert to requested format
        match format {
            HeatmapFormat::Svg => Ok(svg_content.as_bytes().to_vec()),
            HeatmapFormat::Png => self.svg_to_png(&svg_content, theme),
            HeatmapFormat::Jpeg => self.svg_to_jpeg(&svg_content, theme),
            HeatmapFormat::WebP => self.svg_to_webp(&svg_content, theme),
        }
    }

    /// Generate SVG content (internal)
    fn generate_svg(
        &self,
        theme: &heatmap_theme::Model,
        data: &HeatmapData,
        username: Option<&str>,
    ) -> Result<String> {
        let cell_size = theme.cell_size as usize;
        let cell_gap = theme.cell_gap as usize;
        let day_label_width = theme.day_label_width as usize;
        let month_label_height = theme.month_label_height as usize;
        let title_height = theme.title_height as usize;
        let legend_height = theme.legend_height as usize;
        let padding_right = theme.padding_right as usize;
        let padding_bottom = theme.padding_bottom as usize;

        // Calculate dimensions
        let num_weeks = data.weeks.len();
        let num_days = 7;

        let graph_width = num_weeks * (cell_size + cell_gap) - cell_gap;
        let graph_height = num_days * (cell_size + cell_gap) - cell_gap;

        // Total dimensions including all UI elements
        let total_width = day_label_width + graph_width + padding_right;
        let total_height =
            title_height + month_label_height + graph_height + legend_height + padding_bottom;

        // Get color palette
        let mut palette = ColorPalette::from_scheme(&theme.color_scheme);

        // Override with custom colors ONLY if color_scheme is Custom
        if matches!(
            theme.color_scheme,
            heatmap_theme::HeatmapColorScheme::Custom
        ) {
            if let Some(custom_colors_json) = &theme.custom_colors {
                if let Some(colors_array) = custom_colors_json.as_array() {
                    palette.colors = colors_array
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    log::info!("Using custom colors from JSON: {:?}", palette.colors);
                }
            }
        } else {
            log::info!(
                "Using default palette for scheme {:?}: {:?}",
                theme.color_scheme,
                palette.colors
            );
        }

        let mut svg = String::new();

        // Use custom dimensions if provided, otherwise use calculated dimensions
        let (final_width, final_height) = match (theme.heatmap_width, theme.heatmap_height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => {
                // Width specified, calculate height to maintain aspect ratio
                let aspect_ratio = total_height as f64 / total_width as f64;
                (w, (w as f64 * aspect_ratio) as i32)
            }
            (None, Some(h)) => {
                // Height specified, calculate width to maintain aspect ratio
                let aspect_ratio = total_width as f64 / total_height as f64;
                ((h as f64 * aspect_ratio) as i32, h)
            }
            (None, None) => (total_width as i32, total_height as i32),
        };

        // Always use viewBox for proper scaling in browsers
        // If custom dimensions are specified, use preserveAspectRatio="none" to stretch content
        if theme.heatmap_width.is_some() || theme.heatmap_height.is_some() {
            svg.push_str(&format!(
                r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg">"#,
                final_width, final_height, total_width, total_height
            ));
        } else {
            // Include viewBox even for default dimensions to enable CSS scaling
            svg.push_str(&format!(
                r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
                total_width, total_height, total_width, total_height
            ));
        }

        // Background
        svg.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            theme.background_color
        ));

        // Username at top right if enabled (same level as contribution count)
        if theme.show_username {
            if let Some(user) = username {
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="end">@{}</text>"#,
                    total_width - 5, // Position at right with small margin
                    20, // Same level as contribution count
                    theme.font_family,
                    theme.font_size + 2, // Same size as contribution count
                    theme.text_color,
                    user
                ));
            }
        }

        // Title with contribution count
        if theme.show_total_count {
            let title_text = format!("{} contributions in the last year", data.total_count);
            // No need to push down if username is on the right
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                day_label_width,
                20, // Default position
                theme.font_family,
                theme.font_size + 2, // Slightly larger for title
                theme.text_color,
                title_text
            ));
        }

        // Month labels
        if theme.show_month_labels {
            let month_names = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            let mut last_month = -1;

            for (week_idx, week) in data.weeks.iter().enumerate() {
                // Find the first valid day in the week to determine the month
                if let Some(first_day) = week.iter().find(|d| d.count != -1) {
                    let month = first_day.date.month0() as i32; // 0-based month

                    if month != last_month {
                        let x = day_label_width + week_idx * (cell_size + cell_gap);
                        let y = title_height + month_label_height - 3;

                        svg.push_str(&format!(
                            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                            x,
                            y,
                            theme.font_family,
                            theme.font_size,
                            theme.text_color,
                            month_names[month as usize]
                        ));

                        last_month = month;
                    }
                }
            }
        }

        // Day labels (Mon, Wed, Fri)
        if theme.show_day_labels {
            let day_labels = vec![(1, "Mon"), (3, "Wed"), (5, "Fri")];

            for (day_idx, label) in day_labels {
                let y = title_height
                    + month_label_height
                    + day_idx * (cell_size + cell_gap)
                    + cell_size / 2
                    + 3;
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="start">{}</text>"#,
                    5, // Left margin
                    y,
                    theme.font_family,
                    theme.font_size,
                    theme.text_color,
                    label
                ));
            }
        }

        // Draw cells
        for (week_idx, week) in data.weeks.iter().enumerate() {
            for (day_idx, day) in week.iter().enumerate() {
                if day.count == -1 {
                    continue; // Skip placeholder cells
                }

                let x = day_label_width + week_idx * (cell_size + cell_gap);
                let y = title_height + month_label_height + day_idx * (cell_size + cell_gap);

                let color = if day.count == 0 {
                    &theme.empty_cell_color
                } else {
                    palette.get_color_for_count(day.count, data.max_count)
                };

                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" "#,
                    x, y, cell_size, cell_size, theme.cell_border_radius, color
                ));

                if theme.cell_border_width > 0 {
                    svg.push_str(&format!(
                        r#"stroke="{}" stroke-width="{}" "#,
                        theme.cell_border_color, theme.cell_border_width
                    ));
                }

                svg.push_str("/>");
            }
        }

        // Legend at bottom right
        if theme.show_legend {
            let legend_y = title_height + month_label_height + graph_height + 8;
            let legend_start_x =
                total_width - (40 + palette.colors.len() * (cell_size + 3) + 40 + padding_right);

            // "Less" label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="end">{}</text>"#,
                legend_start_x + 30,
                legend_y + cell_size / 2 + 3,
                theme.font_family,
                theme.font_size,
                theme.text_color,
                "Less"
            ));

            // Color squares
            for (i, color) in palette.colors.iter().enumerate() {
                let x = legend_start_x + 35 + i * (cell_size + 3);
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}"/>"#,
                    x, legend_y, cell_size, cell_size, theme.cell_border_radius, color
                ));
            }

            // "More" label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="start">{}</text>"#,
                legend_start_x + 40 + palette.colors.len() * (cell_size + 3),
                legend_y + cell_size / 2 + 3,
                theme.font_family,
                theme.font_size,
                theme.text_color,
                "More"
            ));
        }

        // Watermark at bottom left (clickable link in SVG)
        if theme.show_watermark {
            let watermark_y = title_height + month_label_height + graph_height + 8;
            let watermark_text = "Powered by Hgitmap";
            let watermark_url = "https://github.com/Doublefire-Chen/hgitmap";

            // Wrap in clickable link for SVG format
            svg.push_str(&format!(
                r#"<a href="{}" target="_blank" rel="noopener">"#,
                watermark_url
            ));

            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" opacity="0.6" text-anchor="start">{}</text>"#,
                day_label_width,
                watermark_y + cell_size / 2 + 3,
                theme.font_family,
                theme.font_size - 1, // Slightly smaller than regular font
                theme.text_color,
                watermark_text
            ));

            svg.push_str("</a>");
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Convert SVG to PNG
    fn svg_to_png(&self, svg_content: &str, _theme: &heatmap_theme::Model) -> Result<Vec<u8>> {
        // Create font database and load system fonts
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();

        let opts = usvg::Options::default();

        // Parse the SVG - in usvg 0.37, fonts are resolved during parsing
        let mut tree = usvg::Tree::from_data(svg_content.as_bytes(), &opts)?;

        // Convert text to paths using the font database
        tree.convert_text(&fontdb);

        // Render at 4x resolution for maximum quality
        let scale = 4.0;
        let pixmap_size = tree.size.to_int_size();
        let scaled_width = (pixmap_size.width() as f32 * scale) as u32;
        let scaled_height = (pixmap_size.height() as f32 * scale) as u32;

        let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
            .context("Failed to create pixmap")?;

        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::Tree::from_usvg(&tree).render(transform, &mut pixmap.as_mut());

        let img: RgbaImage =
            ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                .context("Failed to create image buffer")?;

        // Use PNG encoder with best compression (lossless)
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut buffer,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        );
        encoder.write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        )?;

        Ok(buffer)
    }

    /// Convert SVG to JPEG
    fn svg_to_jpeg(&self, svg_content: &str, _theme: &heatmap_theme::Model) -> Result<Vec<u8>> {
        // Create font database and load system fonts
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();

        let opts = usvg::Options::default();

        // Parse the SVG - in usvg 0.37, fonts are resolved during parsing
        let mut tree = usvg::Tree::from_data(svg_content.as_bytes(), &opts)?;

        // Convert text to paths using the font database
        tree.convert_text(&fontdb);

        // Render at 4x resolution for maximum quality
        let scale = 4.0;
        let pixmap_size = tree.size.to_int_size();
        let scaled_width = (pixmap_size.width() as f32 * scale) as u32;
        let scaled_height = (pixmap_size.height() as f32 * scale) as u32;

        let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
            .context("Failed to create pixmap")?;

        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::Tree::from_usvg(&tree).render(transform, &mut pixmap.as_mut());

        // Convert RGBA to RGB for JPEG (no transparency)
        let rgba_img: RgbaImage =
            ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                .context("Failed to create image buffer")?;

        let rgb_img = image::DynamicImage::ImageRgba8(rgba_img).to_rgb8();

        // Use quality 100 for maximum quality (no compression artifacts)
        let mut buffer = Vec::new();
        rgb_img.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Jpeg(100),
        )?;

        Ok(buffer)
    }

    /// Convert SVG to WebP
    fn svg_to_webp(&self, svg_content: &str, _theme: &heatmap_theme::Model) -> Result<Vec<u8>> {
        // Create font database and load system fonts
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();

        let opts = usvg::Options::default();

        // Parse the SVG - in usvg 0.37, fonts are resolved during parsing
        let mut tree = usvg::Tree::from_data(svg_content.as_bytes(), &opts)?;

        // Convert text to paths using the font database
        tree.convert_text(&fontdb);

        // Render at 4x resolution for maximum quality
        let scale = 4.0;
        let pixmap_size = tree.size.to_int_size();
        let scaled_width = (pixmap_size.width() as f32 * scale) as u32;
        let scaled_height = (pixmap_size.height() as f32 * scale) as u32;

        let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
            .context("Failed to create pixmap")?;

        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::Tree::from_usvg(&tree).render(transform, &mut pixmap.as_mut());

        let img: RgbaImage =
            ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                .context("Failed to create image buffer")?;

        // WebP with maximum quality
        let mut buffer = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::WebP,
        )?;

        Ok(buffer)
    }

    /// Get output directory for user's heatmaps
    fn get_output_directory(&self, user_id: uuid::Uuid) -> Result<PathBuf> {
        // Use default path: static/heatmaps/{user_id}
        let base_dir = "static/heatmaps";

        let mut path = PathBuf::from(base_dir);
        path.push(user_id.to_string());

        // Create directory if it doesn't exist
        fs::create_dir_all(&path).context(format!(
            "Failed to create heatmap directory: {}",
            path.display()
        ))?;

        Ok(path)
    }

    /// Get file path for a specific format
    fn get_file_path(
        &self,
        output_dir: &PathBuf,
        theme_slug: &str,
        format: &HeatmapFormat,
    ) -> PathBuf {
        let extension = match format {
            HeatmapFormat::Svg => "svg",
            HeatmapFormat::Png => "png",
            HeatmapFormat::Jpeg => "jpg",
            HeatmapFormat::WebP => "webp",
        };

        output_dir.join(format!("{}.{}", theme_slug, extension))
    }

    /// Calculate SHA-256 hash of file content
    fn calculate_hash(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }
}

fn format_to_string(format: &HeatmapFormat) -> &str {
    match format {
        HeatmapFormat::Svg => "SVG",
        HeatmapFormat::Png => "PNG",
        HeatmapFormat::Jpeg => "JPEG",
        HeatmapFormat::WebP => "WebP",
    }
}
