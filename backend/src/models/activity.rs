use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "activity_type")]
pub enum ActivityType {
    #[sea_orm(string_value = "commit")]
    Commit,
    #[sea_orm(string_value = "repository_created")]
    RepositoryCreated,
    #[sea_orm(string_value = "pull_request")]
    PullRequest,
    #[sea_orm(string_value = "issue")]
    Issue,
    #[sea_orm(string_value = "review")]
    Review,
    #[sea_orm(string_value = "organization_joined")]
    OrganizationJoined,
    #[sea_orm(string_value = "fork")]
    Fork,
    #[sea_orm(string_value = "release")]
    Release,
    #[sea_orm(string_value = "star")]
    Star,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "activities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub git_platform_account_id: Uuid,
    pub activity_type: ActivityType,
    pub activity_date: ChronoDate,
    pub metadata: JsonValue,
    pub repository_name: Option<String>,
    pub repository_url: Option<String>,
    pub is_private_repo: bool,
    pub count: i32,
    pub primary_language: Option<String>,
    pub organization_name: Option<String>,
    pub organization_avatar_url: Option<String>,
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

// Helper structs for metadata deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitActivity {
    pub repositories: Vec<RepositoryCommits>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryCommits {
    pub name: String,
    pub commit_count: i32,
    pub commits: Option<Vec<CommitInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryCreatedActivity {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub is_fork: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationJoinedActivity {
    pub organization: String,
    pub avatar_url: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestActivity {
    pub title: String,
    pub number: i32,
    pub state: String,
    pub repository: String,
    pub url: String,
}
