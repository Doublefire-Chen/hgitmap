use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "heatmap_color_scheme")]
pub enum HeatmapColorScheme {
    #[sea_orm(string_value = "github_green")]
    GitHubGreen,
    #[sea_orm(string_value = "github_blue")]
    GitHubBlue,
    #[sea_orm(string_value = "halloween")]
    Halloween,
    #[sea_orm(string_value = "winter")]
    Winter,
    #[sea_orm(string_value = "ocean")]
    Ocean,
    #[sea_orm(string_value = "sunset")]
    Sunset,
    #[sea_orm(string_value = "forest")]
    Forest,
    #[sea_orm(string_value = "monochrome")]
    Monochrome,
    #[sea_orm(string_value = "rainbow")]
    Rainbow,
    #[sea_orm(string_value = "custom")]
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "theme_mode")]
pub enum ThemeMode {
    #[sea_orm(string_value = "light")]
    Light,
    #[sea_orm(string_value = "dark")]
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "heatmap_format")]
pub enum HeatmapFormat {
    #[sea_orm(string_value = "svg")]
    Svg,
    #[sea_orm(string_value = "png")]
    Png,
    #[sea_orm(string_value = "jpeg")]
    Jpeg,
    #[sea_orm(string_value = "webp")]
    WebP,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "heatmap_themes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,

    // Theme identification
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_default: bool,

    // Light/Dark mode
    pub theme_mode: ThemeMode,

    // Color configuration
    pub color_scheme: HeatmapColorScheme,
    pub custom_colors: Option<Json>,

    // Theme colors
    pub background_color: String,
    pub border_color: String,
    pub text_color: String,
    pub empty_cell_color: String,

    // Cell/Rectangle styling
    pub cell_size: i32,
    pub cell_gap: i32,
    pub cell_border_radius: i32,
    pub cell_border_width: i32,
    pub cell_border_color: String,

    // Overall heatmap dimensions
    pub heatmap_width: Option<i32>,
    pub heatmap_height: Option<i32>,

    // Padding
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,

    // Layout spacing settings
    pub day_label_width: i32,
    pub month_label_height: i32,
    pub title_height: i32,
    pub legend_height: i32,

    // Display options
    pub show_month_labels: bool,
    pub show_day_labels: bool,
    pub show_legend: bool,
    pub show_total_count: bool,
    pub show_username: bool,
    pub show_watermark: bool,

    // Font settings
    pub font_family: String,
    pub font_size: i32,

    // Legend settings
    pub legend_position: String,

    // Output formats (array of formats to generate)
    pub output_formats: Vec<HeatmapFormat>,

    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::generated_heatmap::Entity")]
    GeneratedHeatmaps,
    #[sea_orm(has_many = "super::heatmap_generation_job::Entity")]
    GenerationJobs,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::generated_heatmap::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GeneratedHeatmaps.def()
    }
}

impl Related<super::heatmap_generation_job::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GenerationJobs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
