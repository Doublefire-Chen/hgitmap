use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "heatmap_generation_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,

    // Update interval in minutes
    pub update_interval_minutes: i32,

    // Auto-generation control
    pub auto_generation_enabled: bool,

    // Date range for heatmap (in days)
    pub date_range_days: i32,

    // Privacy settings
    pub include_private_contributions: bool,

    // Storage path customization
    pub storage_path: Option<String>,

    // Scheduling tracking
    pub last_scheduled_generation_at: Option<ChronoDateTimeUtc>,
    pub next_scheduled_generation_at: Option<ChronoDateTimeUtc>,

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
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
