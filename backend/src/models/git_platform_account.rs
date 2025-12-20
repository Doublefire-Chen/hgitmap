use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "git_platform")]
pub enum GitPlatform {
    #[sea_orm(string_value = "github")]
    GitHub,
    #[sea_orm(string_value = "gitea")]
    Gitea,
    #[sea_orm(string_value = "gitlab")]
    GitLab,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "git_platform_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform_type: GitPlatform,
    pub platform_username: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub platform_url: Option<String>,
    pub is_active: bool,
    pub last_synced_at: Option<ChronoDateTimeUtc>,
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
    #[sea_orm(has_many = "super::contribution::Entity")]
    Contributions,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::contribution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contributions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
