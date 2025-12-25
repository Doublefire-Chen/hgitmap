use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::git_platform_account::GitPlatform;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "oauth_states")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub state_token: String,
    pub user_id: Uuid,
    pub platform: GitPlatform,
    pub created_at: ChronoDateTimeUtc,
    pub expires_at: ChronoDateTimeUtc,
    pub instance_url: Option<String>, // For self-hosted instances (Gitea/GitLab)
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
