use super::{Activity, ActivityType, Contribution, ContributionType, GitPlatform, PlatformConfig, Repository, UserInfo};
use crate::utils::http_client::create_http_client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc, NaiveDate};
use serde::Deserialize;
use std::collections::HashMap;

pub struct GitLabClient;

// GitLab API response structures

#[derive(Debug, Deserialize)]
struct GitLabUser {
    id: i64,
    username: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabEvent {
    action_name: String,
    target_type: Option<String>,
    target_title: Option<String>,
    created_at: String,
    push_data: Option<GitLabPushData>,
}

#[derive(Debug, Deserialize)]
struct GitLabPushData {
    commit_count: i32,
    ref_type: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    name: String,
    path_with_namespace: String,
    visibility: String,
    web_url: String,
    created_at: String,
}

impl GitLabClient {
    pub fn new() -> Self {
        Self
    }

    /// Fetch user profile data from GitLab
    pub async fn fetch_user_profile(
        &self,
        config: &PlatformConfig,
        token: &str,
    ) -> Result<serde_json::Value> {
        let client = create_http_client();

        log::info!("👤 Fetching GitLab profile");

        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Failed to fetch GitLab profile: status {}, error: {}", status, error_text);
            return Err(anyhow!("Failed to fetch GitLab profile: status {}", status));
        }

        let profile: serde_json::Value = response.json().await?;

        log::info!("✅ Fetched GitLab profile");

        Ok(profile)
    }

    /// Fetch user's contribution events from GitLab
    async fn fetch_user_events(
        &self,
        config: &PlatformConfig,
        user_id: i64,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<GitLabEvent>> {
        let client = create_http_client();
        let mut all_events = Vec::new();
        let mut page = 1;
        let per_page = 100;

        log::info!("📡 Fetching GitLab events for user ID: {}", user_id);

        loop {
            // Temporarily remove date filters to test if events exist
            let url = format!(
                "{}/users/{}/events?per_page={}&page={}",
                config.api_base_url,
                user_id,
                per_page,
                page
            );

            log::debug!("Requesting page {}: {}", page, url);
            log::info!("🌐 GitLab API URL (NO DATE FILTER TEST): {}", url);

            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("accept", "application/json")
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                log::error!("Failed to fetch GitLab events: status {}, error: {}", status, error_text);
                return Err(anyhow!("Failed to fetch GitLab events: status {}", status));
            }

            let events: Vec<GitLabEvent> = response.json().await?;
            let event_count = events.len();

            log::debug!("📥 Fetched {} events from page {}", event_count, page);

            // Log first event for debugging
            if !events.is_empty() {
                log::info!("📋 Sample event action_name: {:?}", events[0].action_name);
            }

            if events.is_empty() {
                break;
            }

            all_events.extend(events);

            // If we got fewer than per_page items, we're done
            if event_count < per_page {
                break;
            }

            page += 1;

            // Safety limit to prevent infinite loops
            if page > 100 {
                log::warn!("⚠️  Reached page limit (100), stopping pagination");
                break;
            }
        }

        log::info!("✅ Fetched total of {} events from GitLab", all_events.len());

        Ok(all_events)
    }
}

#[async_trait]
impl GitPlatform for GitLabClient {
    async fn validate_token(&self, config: &PlatformConfig, token: &str) -> Result<UserInfo> {
        let client = create_http_client();

        log::info!("🔍 Validating GitLab token");

        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Failed to validate GitLab token: status {}, error: {}", status, error_text);
            return Err(anyhow!("Failed to validate GitLab token: status {}", status));
        }

        let user: GitLabUser = response.json().await?;

        log::info!("✅ GitLab token validated for user: {}", user.username);

        Ok(UserInfo {
            username: user.username,
            id: user.id.to_string(),
            email: user.email,
            avatar_url: user.avatar_url,
        })
    }

    async fn fetch_contributions(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Contribution>> {
        log::info!("🔍 Fetching GitLab contributions for {}", username);

        // First, get user info to get the user ID
        let user_info = self.validate_token(config, token).await?;
        let user_id: i64 = user_info.id.parse()
            .map_err(|e| anyhow!("Failed to parse user ID: {}", e))?;

        // Fetch events
        let events = self.fetch_user_events(config, user_id, token, from, to).await?;

        // Group contributions by date and repository
        let mut contributions_map: HashMap<(NaiveDate, Option<String>), i32> = HashMap::new();

        for event in events {
            // Parse the created_at timestamp
            let event_date = match DateTime::parse_from_rfc3339(&event.created_at) {
                Ok(dt) => dt.with_timezone(&Utc).date_naive(),
                Err(e) => {
                    log::warn!("Failed to parse event date: {}", e);
                    continue;
                }
            };

            // Filter by date range
            if event_date < from.date_naive() || event_date > to.date_naive() {
                continue;
            }

            // Only count push events for contributions
            // GitLab API returns action_name as "pushed to" or "pushed new"
            if event.action_name.starts_with("pushed") {
                if let Some(push_data) = &event.push_data {
                    let count = push_data.commit_count;

                    // Try to extract repository name from event
                    let repo_name = event.target_title.clone();

                    let key = (event_date, repo_name.clone());
                    *contributions_map.entry(key).or_insert(0) += count;

                    log::debug!("📊 Push event: {} commits on {} to {:?}", count, event_date, repo_name);
                } else {
                    log::warn!("⚠️  Push event has no push_data: action_name={}, target_title={:?}", event.action_name, event.target_title);
                }
            } else {
                log::debug!("⏭️  Skipping non-push event: action_name={}", event.action_name);
            }
        }

        // Convert map to Vec<Contribution>
        let contributions: Vec<Contribution> = contributions_map
            .into_iter()
            .map(|((date, repo_name), count)| Contribution {
                date,
                count,
                repository_name: repo_name,
                is_private: false, // GitLab events API doesn't easily expose visibility
                contribution_type: ContributionType::Commit,
            })
            .collect();

        log::info!("✅ Processed {} contribution entries from GitLab", contributions.len());

        Ok(contributions)
    }

    async fn fetch_repositories(
        &self,
        config: &PlatformConfig,
        token: &str,
    ) -> Result<Vec<Repository>> {
        let client = create_http_client();
        let mut all_repos = Vec::new();
        let mut page = 1;
        let per_page = 100;

        log::info!("📡 Fetching GitLab repositories");

        loop {
            let url = format!(
                "{}/projects?membership=true&per_page={}&page={}",
                config.api_base_url,
                per_page,
                page
            );

            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("accept", "application/json")
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                return Err(anyhow!("Failed to fetch GitLab repositories: status {}", status));
            }

            let repos: Vec<GitLabProject> = response.json().await?;
            let repo_count = repos.len();

            if repos.is_empty() {
                break;
            }

            for project in repos {
                all_repos.push(Repository {
                    name: project.name,
                    full_name: project.path_with_namespace.clone(),
                    is_private: project.visibility != "public",
                    url: project.web_url,
                });
            }

            if repo_count < per_page {
                break;
            }

            page += 1;

            if page > 100 {
                log::warn!("⚠️  Reached page limit, stopping pagination");
                break;
            }
        }

        log::info!("✅ Fetched {} repositories from GitLab", all_repos.len());

        Ok(all_repos)
    }

    async fn fetch_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        log::info!("🔍 Fetching GitLab activities for {}", username);

        // Get user info
        let user_info = self.validate_token(config, token).await?;
        let user_id: i64 = user_info.id.parse()
            .map_err(|e| anyhow!("Failed to parse user ID: {}", e))?;

        // Fetch events
        let events = self.fetch_user_events(config, user_id, token, from, to).await?;

        let mut activities = Vec::new();

        for event in events {
            let event_date = match DateTime::parse_from_rfc3339(&event.created_at) {
                Ok(dt) => dt.with_timezone(&Utc).date_naive(),
                Err(e) => {
                    log::warn!("Failed to parse event date: {}", e);
                    continue;
                }
            };

            // Filter by date range
            if event_date < from.date_naive() || event_date > to.date_naive() {
                continue;
            }

            // Map GitLab events to our ActivityType
            let activity_type = match event.action_name.as_str() {
                "pushed" => {
                    // GitLab API action_name is "pushed" (not "pushed to" or "pushed new")
                    if let Some(push_data) = &event.push_data {
                        activities.push(Activity {
                            activity_type: ActivityType::Commit,
                            date: event_date,
                            metadata: serde_json::json!({
                                "total_count": push_data.commit_count,
                                "ref": push_data.ref_name,
                                "ref_type": push_data.ref_type,
                            }),
                            repository_name: event.target_title.clone(),
                            repository_url: None, // Would need additional API call
                            is_private: false,
                            count: push_data.commit_count,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                    continue;
                },
                "opened" if event.target_type.as_deref() == Some("MergeRequest") => {
                    ActivityType::PullRequest
                },
                "opened" if event.target_type.as_deref() == Some("Issue") => {
                    ActivityType::Issue
                },
                "commented on" => {
                    ActivityType::Review
                },
                _ => continue, // Skip other event types
            };

            // Create activity for non-commit events
            if activity_type != ActivityType::Commit {
                activities.push(Activity {
                    activity_type,
                    date: event_date,
                    metadata: serde_json::json!({
                        "title": event.target_title,
                        "type": event.target_type,
                        "action": event.action_name,
                    }),
                    repository_name: event.target_title.clone(),
                    repository_url: None,
                    is_private: false,
                    count: 1,
                    primary_language: None,
                    organization_name: None,
                    organization_avatar_url: None,
                });
            }
        }

        log::info!("✅ Processed {} activities from GitLab", activities.len());

        Ok(activities)
    }

    async fn fetch_repository_creation_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        let client = create_http_client();
        let mut all_activities = Vec::new();
        let mut page = 1;
        let per_page = 100;

        log::info!("📡 Fetching GitLab repository creation activities for {}", username);

        loop {
            let url = format!(
                "{}/projects?membership=true&per_page={}&page={}",
                config.api_base_url,
                per_page,
                page
            );

            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("accept", "application/json")
                .send()
                .await?;

            if !response.status().is_success() {
                break;
            }

            let projects: Vec<GitLabProject> = response.json().await?;
            let project_count = projects.len();

            if projects.is_empty() {
                break;
            }

            for project in projects {
                // Parse created_at
                let created_at = match DateTime::parse_from_rfc3339(&project.created_at) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(e) => {
                        log::warn!("Failed to parse project created_at: {}", e);
                        continue;
                    }
                };

                // Filter by date range
                if created_at < from || created_at > to {
                    continue;
                }

                all_activities.push(Activity {
                    activity_type: ActivityType::RepositoryCreated,
                    date: created_at.date_naive(),
                    metadata: serde_json::json!({
                        "name": project.path_with_namespace,
                        "created_at": project.created_at,
                        "is_private": project.visibility != "public",
                    }),
                    repository_name: Some(project.path_with_namespace.clone()),
                    repository_url: Some(project.web_url),
                    is_private: project.visibility != "public",
                    count: 1,
                    primary_language: None,
                    organization_name: None,
                    organization_avatar_url: None,
                });
            }

            if project_count < per_page {
                break;
            }

            page += 1;

            if page > 100 {
                break;
            }
        }

        log::info!("✅ Found {} repository creation activities from GitLab", all_activities.len());

        Ok(all_activities)
    }
}
