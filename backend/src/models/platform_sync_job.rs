use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "sync_job_status")]
pub enum SyncJobStatus {
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
#[sea_orm(table_name = "platform_sync_jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform_account_id: Uuid,

    pub status: SyncJobStatus,

    // Sync parameters
    pub sync_all_years: bool,
    pub specific_year: Option<i32>,
    pub sync_contributions: bool,
    pub sync_activities: bool,
    pub sync_profile: bool,

    // Job scheduling
    pub scheduled_at: ChronoDateTimeUtc,
    pub started_at: Option<ChronoDateTimeUtc>,
    pub completed_at: Option<ChronoDateTimeUtc>,

    // Result tracking
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,

    // Progress tracking
    pub contributions_synced: Option<i32>,
    pub activities_synced: Option<i32>,
    pub years_completed: Option<i32>,
    pub total_years: Option<i32>,

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
        belongs_to = "super::git_platform_account::Entity",
        from = "Column::PlatformAccountId",
        to = "super::git_platform_account::Column::Id"
    )]
    PlatformAccount,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::git_platform_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlatformAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
