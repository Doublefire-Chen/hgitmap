pub mod github;
pub mod gitea;
pub mod gitlab;

pub use github::GitHubClient;
pub use gitea::GiteaClient;
pub use gitlab::GitLabClient;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use serde_json::Value as JsonValue;

/// Represents a contribution event from a git platform
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contribution {
    pub date: chrono::NaiveDate,
    pub count: i32,
    pub repository_name: Option<String>,
    pub is_private: bool,
    pub contribution_type: ContributionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContributionType {
    Commit,
    PullRequest,
    Issue,
    Review,
    Other,
}

/// Represents an activity (for activity timeline)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub activity_type: ActivityType,
    pub date: chrono::NaiveDate,
    pub metadata: JsonValue,
    pub repository_name: Option<String>,
    pub repository_url: Option<String>,
    pub is_private: bool,
    pub count: i32,
    pub primary_language: Option<String>,
    pub organization_name: Option<String>,
    pub organization_avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Commit,
    RepositoryCreated,
    PullRequest,
    Issue,
    Review,
    OrganizationJoined,
    Fork,
    Release,
    Star,
}

/// User information returned from platform APIs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub id: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Repository information from platform APIs
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub is_private: bool,
    pub url: String,
}

/// Configuration for a specific platform instance
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PlatformConfig {
    pub platform: String,
    pub instance_url: String,
    pub api_base_url: String,
}

impl PlatformConfig {
    /// Create a GitHub.com configuration
    pub fn github() -> Self {
        Self {
            platform: "github".to_string(),
            instance_url: "https://github.com".to_string(),
            api_base_url: "https://api.github.com".to_string(),
        }
    }

    /// Create a GitLab.com configuration
    pub fn gitlab() -> Self {
        Self {
            platform: "gitlab".to_string(),
            instance_url: "https://gitlab.com".to_string(),
            api_base_url: "https://gitlab.com/api/v4".to_string(),
        }
    }

    /// Create a custom GitLab instance configuration
    pub fn gitlab_custom(instance_url: &str) -> Self {
        Self {
            platform: "gitlab".to_string(),
            instance_url: instance_url.to_string(),
            api_base_url: format!("{}/api/v4", instance_url.trim_end_matches('/')),
        }
    }

    /// Create a custom Gitea instance configuration
    pub fn gitea_custom(instance_url: &str) -> Self {
        Self {
            platform: "gitea".to_string(),
            instance_url: instance_url.to_string(),
            api_base_url: format!("{}/api/v1", instance_url.trim_end_matches('/')),
        }
    }
}

/// Trait that all git platform integrations must implement
#[async_trait]
pub trait GitPlatform: Send + Sync {
    /// Fetch contributions for a user within a date range
    async fn fetch_contributions(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Contribution>>;

    /// Validate an access token and return user information
    async fn validate_token(&self, config: &PlatformConfig, token: &str) -> Result<UserInfo>;

    /// Fetch repositories accessible with the given token
    #[allow(dead_code)]
    async fn fetch_repositories(
        &self,
        config: &PlatformConfig,
        token: &str,
    ) -> Result<Vec<Repository>>;

    /// Fetch user activities (for activity timeline)
    async fn fetch_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>>;

    /// Fetch repository creation activities (works for all history, not limited by Events API)
    async fn fetch_repository_creation_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>>;
}
