use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "generation_job_status")]
pub enum GenerationJobStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "processing")]
    Processing,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "heatmap_generation_jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub theme_id: Option<Uuid>,

    pub status: GenerationJobStatus,

    // Job scheduling
    pub scheduled_at: ChronoDateTimeUtc,
    pub started_at: Option<ChronoDateTimeUtc>,
    pub completed_at: Option<ChronoDateTimeUtc>,

    // Result tracking
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,

    // Job metadata
    pub is_manual: bool,
    pub priority: i32,

    pub created_at: ChronoDateTimeUtc,
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
