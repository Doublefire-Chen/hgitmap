use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "generated_heatmaps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub theme_id: Uuid,

    // Format of this generated file
    pub format: super::heatmap_theme::HeatmapFormat,

    // File information
    pub file_path: String,
    pub file_size_bytes: Option<i64>,
    pub file_hash: Option<String>,

    // Generation metadata
    pub generated_at: ChronoDateTimeUtc,
    pub generation_duration_ms: Option<i32>,

    // Data snapshot info
    pub contribution_count: i32,
    pub date_range_start: ChronoDate,
    pub date_range_end: ChronoDate,

    // Access tracking
    pub access_count: i32,
    pub last_accessed_at: Option<ChronoDateTimeUtc>,

    // Status
    pub is_valid: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::heatmap_theme::Entity",
        from = "Column::ThemeId",
        to = "super::heatmap_theme::Column::Id"
    )]
    Theme,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::heatmap_theme::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Theme.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
