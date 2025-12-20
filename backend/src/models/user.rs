use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::git_platform_account::Entity")]
    GitPlatformAccounts,
    #[sea_orm(has_one = "super::user_setting::Entity")]
    UserSettings,
    #[sea_orm(has_many = "super::api_token::Entity")]
    ApiTokens,
}

impl Related<super::git_platform_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GitPlatformAccounts.def()
    }
}

impl Related<super::user_setting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSettings.def()
    }
}

impl Related<super::api_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
