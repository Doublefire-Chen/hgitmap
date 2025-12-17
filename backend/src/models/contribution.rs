use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contributions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub git_platform_account_id: Uuid,
    pub contribution_date: ChronoDate,
    pub count: i32,
    pub repository_name: Option<String>,
    pub is_private_repo: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::git_platform_account::Entity",
        from = "Column::GitPlatformAccountId",
        to = "super::git_platform_account::Column::Id"
    )]
    GitPlatformAccount,
}

impl Related<super::git_platform_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GitPlatformAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
