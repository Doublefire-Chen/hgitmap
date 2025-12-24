use super::{Activity, ActivityType, Contribution, ContributionType, GitPlatform, PlatformConfig, Repository, UserInfo};
use crate::utils::http_client::create_http_client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct GiteaClient;

// Gitea heatmap API response format
#[derive(Debug, Deserialize)]
struct GiteaHeatmapEntry {
    timestamp: i64,  // Unix timestamp in seconds
    contributions: i64,
}

impl GiteaClient {
    pub fn new() -> Self {
        Self
    }

    /// Fetch user profile data from Gitea
    pub async fn fetch_user_profile(
        &self,
        config: &PlatformConfig,
        token: &str,
    ) -> Result<serde_json::Value> {
        let client = create_http_client();

        log::info!("👤 Fetching Gitea profile");

        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("token {}", token))
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Failed to fetch Gitea profile: status {}", status));
        }

        let profile: serde_json::Value = response.json().await?;

        log::info!("✅ Fetched Gitea profile");

        Ok(profile)
    }
}

#[async_trait]
impl GitPlatform for GiteaClient {
    async fn fetch_contributions(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Contribution>> {
        let client = create_http_client();

        log::info!("🔍 Fetching Gitea contributions for {} using heatmap API", username);

        // Use Gitea's public heatmap API endpoint with authentication
        // This should include private repos if token has proper permissions
        let heatmap_url = format!("{}/users/{}/heatmap", config.api_base_url, username);
        log::info!("📡 Requesting: {}", heatmap_url);

        // Use Gitea's dedicated heatmap API endpoint
        // This matches exactly what Gitea displays on the user's profile
        let response = client
            .get(&heatmap_url)
            .header("Authorization", format!("token {}", token))
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Failed to fetch Gitea heatmap: status {}, error: {}", status, error_text);
            return Err(anyhow!("Failed to fetch Gitea heatmap: status {}", status));
        }

        let heatmap_entries: Vec<GiteaHeatmapEntry> = response.json().await?;
        log::info!("📥 Fetched {} heatmap entries from Gitea", heatmap_entries.len());

        if heatmap_entries.len() > 0 {
            log::info!("First entry: timestamp={}, contributions={}",
                heatmap_entries[0].timestamp, heatmap_entries[0].contributions);
            log::info!("Last entry: timestamp={}, contributions={}",
                heatmap_entries[heatmap_entries.len()-1].timestamp,
                heatmap_entries[heatmap_entries.len()-1].contributions);
        }

        log::info!("Date range filter: from={}, to={}", from, to);

        // Convert heatmap entries to contributions and filter by date range
        // Use HashMap to aggregate contributions by date (sum up multiple entries per day)
        let mut contributions_by_date: HashMap<chrono::NaiveDate, i64> = HashMap::new();
        let mut filtered_out = 0;

        for entry in heatmap_entries {
            // Convert Unix timestamp to DateTime
            let datetime = DateTime::from_timestamp(entry.timestamp, 0)
                .ok_or_else(|| anyhow!("Invalid timestamp: {}", entry.timestamp))?;

            // Get the naive date in UTC
            let date = datetime.naive_utc().date();

            // Filter by date range
            if datetime < from || datetime > to {
                filtered_out += 1;
                log::debug!("Filtered out: timestamp={}, date={} (outside range)", entry.timestamp, date);
                continue;
            }

            log::debug!("Heatmap entry: timestamp={}, date={}, contributions={}",
                entry.timestamp, date, entry.contributions);

            // Aggregate contributions by date (sum if date already exists)
            *contributions_by_date.entry(date).or_insert(0) += entry.contributions;
        }

        log::info!("📊 Filtered out {} entries, aggregated into {} unique dates",
            filtered_out, contributions_by_date.len());

        // Now fetch activities to get repository names
        log::info!("📦 Fetching activities to get repository names for contributions...");
        let mut repo_by_date: HashMap<chrono::NaiveDate, (String, bool)> = HashMap::new();

        let mut page = 1;
        let per_page = 50;
        let mut total_activities_fetched = 0;

        // Fetch activities with pagination using /users/{username}/activities/feeds
        // Note: Gitea does NOT have a /user/activities/feeds endpoint (even in latest versions)
        // The only available endpoint is /users/{username}/activities/feeds
        // This endpoint returns activities with authentication, including private repo activities
        loop {
            let response = client
                .get(&format!("{}/users/{}/activities/feeds", config.api_base_url, username))
                .header("Authorization", format!("token {}", token))
                .header("accept", "application/json")
                .query(&[("page", &page.to_string()), ("limit", &per_page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                log::warn!("Failed to fetch activities for repo names: status {}", response.status());
                break;
            }

            let activities: Vec<GiteaActivity> = response.json().await?;

            if activities.is_empty() {
                break;
            }

            total_activities_fetched += activities.len();

            // Process activities to extract repository info by date
            for activity in activities {
                if let Ok(activity_date) = chrono::DateTime::parse_from_rfc3339(&activity.created) {
                    let activity_date_utc = activity_date.with_timezone(&chrono::Utc);

                    // Filter by date range
                    if activity_date_utc < from || activity_date_utc > to {
                        continue;
                    }

                    let naive_date = activity_date.naive_utc().date();

                    // Only process commit-related activities
                    if matches!(activity.op_type.as_str(), "commit_repo" | "push") {
                        if let Some(ref repo) = activity.repo {
                            let repo_full_name = repo.full_name.clone()
                                .unwrap_or_else(|| format!("{}/{}", repo.owner.login, repo.name));

                            // Store the most recent repo for each date
                            // (we can't track all repos perfectly from activities, but this gives us something)
                            if !repo_by_date.contains_key(&naive_date) {
                                repo_by_date.insert(naive_date, (repo_full_name, repo.private));
                            }
                        }
                    }
                }
            }

            page += 1;

            // Limit to 20 pages for performance
            if page > 20 {
                break;
            }
        }

        log::info!("📦 Fetched {} activities total, found repo names for {} dates",
            total_activities_fetched, repo_by_date.len());

        // Convert to contributions - now with proper repository tracking
        // For dates with repo information from activities, create separate entries per repo
        // For dates without repo info, create a single aggregate entry
        let mut contributions: Vec<Contribution> = Vec::new();

        // First, create a map to track contributions by (date, repo)
        let mut contrib_by_date_repo: HashMap<(chrono::NaiveDate, Option<String>), (i64, bool)> = HashMap::new();

        // For each date with contributions
        for (date, total_count) in &contributions_by_date {
            // Check if we have repo information for this date
            if let Some((repo_name, is_private)) = repo_by_date.get(date) {
                // We have ONE repo for this date - use it
                contrib_by_date_repo.insert(
                    (*date, Some(repo_name.clone())),
                    (*total_count, *is_private)
                );
            } else {
                // No repo information - create aggregate entry
                contrib_by_date_repo.insert(
                    (*date, None),
                    (*total_count, false)
                );
            }
        }

        // Convert to Vec<Contribution>
        for ((date, repo_name), (count, is_private)) in contrib_by_date_repo {
            contributions.push(Contribution {
                date,
                count: count as i32,
                repository_name: repo_name,
                is_private,
                contribution_type: ContributionType::Commit,
            });
        }

        // Sort by date
        contributions.sort_by(|a, b| a.date.cmp(&b.date));

        let total: i32 = contributions.iter().map(|c| c.count).sum();
        log::info!("📊 Collected {} contributions across {} entries (total count: {})",
            total, contributions.len(), total);

        Ok(contributions)
    }

    async fn validate_token(&self, config: &PlatformConfig, token: &str) -> Result<UserInfo> {
        let client = create_http_client();

        log::info!("🔑 Validating Gitea token");

        // Use Gitea's user endpoint to validate token and get user info
        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("token {}", token))
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Invalid Gitea token: status {}", status));
        }

        let user: GiteaUser = response.json().await?;

        log::info!("✅ Validated token for user: {}", user.login);

        Ok(UserInfo {
            username: user.login,
            id: user.id.to_string(),
            email: user.email,
            avatar_url: Some(user.avatar_url),
        })
    }

    async fn fetch_repositories(
        &self,
        config: &PlatformConfig,
        token: &str,
    ) -> Result<Vec<Repository>> {
        let client = create_http_client();

        log::info!("📦 Fetching Gitea repositories");

        let mut all_repos = Vec::new();
        let mut page = 1;
        let per_page = 50;

        // Fetch user repositories with pagination
        loop {
            let response = client
                .get(&format!("{}/user/repos", config.api_base_url))
                .header("Authorization", format!("token {}", token))
                .header("accept", "application/json")
                .query(&[("page", &page.to_string()), ("limit", &per_page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                return Err(anyhow!("Failed to fetch repositories: status {}", status));
            }

            let repos: Vec<GiteaRepo> = response.json().await?;

            log::info!("📥 Fetched {} repositories (page {})", repos.len(), page);

            if repos.is_empty() {
                break;
            }

            all_repos.extend(repos);
            page += 1;

            // Limit to 20 pages
            if page > 20 {
                break;
            }
        }

        log::info!("📊 Total repositories fetched: {}", all_repos.len());

        Ok(all_repos
            .into_iter()
            .map(|repo| Repository {
                name: repo.name.clone(),
                full_name: repo.full_name,
                is_private: repo.private,
                url: repo.html_url,
            })
            .collect())
    }

    async fn fetch_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        let client = create_http_client();

        log::info!("🔍 Fetching Gitea activities for {} from {} to {}", username, from, to);

        let mut all_activities = Vec::new();
        let mut page = 1;
        let per_page = 50;

        // Fetch user activities with pagination using /users/{username}/activities/feeds
        // Note: Gitea does NOT have a /user/activities/feeds endpoint
        // The only available endpoint is /users/{username}/activities/feeds
        loop {
            let response = client
                .get(&format!("{}/users/{}/activities/feeds", config.api_base_url, username))
                .header("Authorization", format!("token {}", token))
                .header("accept", "application/json")
                .query(&[("page", &page.to_string()), ("limit", &per_page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                return Err(anyhow!("Failed to fetch Gitea activities: status {}", status));
            }

            let activities: Vec<GiteaActivity> = response.json().await?;

            log::info!("📥 Fetched {} activities from Gitea (page {})", activities.len(), page);

            if activities.is_empty() {
                break;
            }

            all_activities.extend(activities);
            page += 1;

            // Limit to 20 pages
            if page > 20 {
                break;
            }
        }

        log::info!("📊 Total activities fetched from Gitea: {}", all_activities.len());

        // Process activities and convert to our Activity format
        let mut activities = Vec::new();
        let mut commit_groups: HashMap<String, Vec<GiteaActivity>> = HashMap::new();

        for activity in all_activities {
            // Parse activity date
            let activity_date = chrono::DateTime::parse_from_rfc3339(&activity.created)
                .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
            let activity_date_utc = activity_date.with_timezone(&chrono::Utc);

            // Filter by date range
            if activity_date_utc < from || activity_date_utc > to {
                continue;
            }

            let naive_date = activity_date.naive_utc().date();

            match activity.op_type.as_str() {
                "commit_repo" | "push" => {
                    // Group commits by date and repository
                    if let Some(ref repo) = activity.repo {
                        let key = format!("{}_{}", naive_date, repo.full_name.as_ref().unwrap_or(&repo.name));
                        commit_groups.entry(key).or_insert_with(Vec::new).push(activity);
                    }
                }
                "create_repo" => {
                    if let Some(ref repo) = activity.repo {
                        let repo_full_name = format!("{}/{}", repo.owner.login, repo.name);

                        activities.push(Activity {
                            activity_type: ActivityType::RepositoryCreated,
                            date: naive_date,
                            metadata: json!({
                                "name": repo_full_name,
                                "description": repo.description,
                                "created_at": activity.created,
                            }),
                            repository_name: Some(repo_full_name.clone()),
                            repository_url: repo.html_url.clone(),
                            is_private: repo.private,
                            count: 1,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                }
                "create_pull_request" => {
                    if let Some(ref repo) = activity.repo {
                        let repo_full_name = format!("{}/{}", repo.owner.login, repo.name);

                        activities.push(Activity {
                            activity_type: ActivityType::PullRequest,
                            date: naive_date,
                            metadata: json!({
                                "repository": repo_full_name,
                                "content": activity.content,
                            }),
                            repository_name: Some(repo_full_name),
                            repository_url: repo.html_url.clone(),
                            is_private: repo.private,
                            count: 1,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                }
                "create_issue" => {
                    if let Some(ref repo) = activity.repo {
                        let repo_full_name = format!("{}/{}", repo.owner.login, repo.name);

                        activities.push(Activity {
                            activity_type: ActivityType::Issue,
                            date: naive_date,
                            metadata: json!({
                                "repository": repo_full_name,
                                "content": activity.content,
                            }),
                            repository_name: Some(repo_full_name),
                            repository_url: repo.html_url.clone(),
                            is_private: repo.private,
                            count: 1,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                }
                _ => {}
            }
        }

        // Process grouped commits
        for (key, events) in commit_groups {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() < 2 {
                continue;
            }

            let date_str = parts[0];
            let repo_name = parts[1..].join("_");

            let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Utc::now().naive_utc().date());

            let total_commits = events.len() as i32;

            let first_event = &events[0];
            let is_private = first_event.repo.as_ref().map(|r| r.private).unwrap_or(false);
            let repo_url = first_event.repo.as_ref().and_then(|r| r.html_url.clone());

            activities.push(Activity {
                activity_type: ActivityType::Commit,
                date: naive_date,
                metadata: json!({
                    "repository": repo_name,
                    "total_count": total_commits,
                }),
                repository_name: Some(repo_name.clone()),
                repository_url: repo_url,
                is_private,
                count: total_commits,
                primary_language: None,
                organization_name: None,
                organization_avatar_url: None,
            });
        }

        // Sort by date descending
        activities.sort_by(|a, b| b.date.cmp(&a.date));

        log::info!("📊 Processed {} activities", activities.len());

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

        log::info!("📦 Fetching repository creation activities for {}", username);

        // Fetch all user repositories
        let repos = self.fetch_repositories(config, token).await?;

        let mut activities = Vec::new();

        // For each repository, we need to get its creation date
        // Unfortunately, the /user/repos endpoint doesn't always include created_at
        // We'll need to fetch each repo individually
        for repo in repos {
            // Extract owner and repo name from full_name
            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }

            let owner = parts[0];
            let repo_name = parts[1];

            // Fetch detailed repo info
            let response = client
                .get(&format!("{}/repos/{}/{}", config.api_base_url, owner, repo_name))
                .header("Authorization", format!("token {}", token))
                .header("accept", "application/json")
                .send()
                .await;

            if let Ok(resp) = response {
                if resp.status().is_success() {
                    if let Ok(repo_detail) = resp.json::<GiteaRepo>().await {
                        if let Some(created_at) = repo_detail.created_at {
                            let created_date = chrono::DateTime::parse_from_rfc3339(&created_at)
                                .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
                            let created_date_utc = created_date.with_timezone(&chrono::Utc);

                            // Filter by date range
                            if created_date_utc >= from && created_date_utc <= to {
                                let naive_date = created_date.naive_utc().date();

                                activities.push(Activity {
                                    activity_type: ActivityType::RepositoryCreated,
                                    date: naive_date,
                                    metadata: json!({
                                        "name": repo.full_name,
                                        "description": repo_detail.description,
                                        "created_at": created_at,
                                    }),
                                    repository_name: Some(repo.full_name.clone()),
                                    repository_url: Some(repo.url),
                                    is_private: repo.is_private,
                                    count: 1,
                                    primary_language: None,
                                    organization_name: None,
                                    organization_avatar_url: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        log::info!("📦 Fetched {} repository creation activities", activities.len());

        Ok(activities)
    }
}

// Gitea API response types

#[derive(Debug, Deserialize)]
struct GiteaUser {
    login: String,
    id: u64,
    email: Option<String>,
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
struct GiteaRepo {
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
    description: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GiteaActivity {
    #[serde(rename = "op_type")]
    op_type: String,
    created: String,
    repo: Option<GiteaActivityRepo>,
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GiteaActivityRepo {
    name: String,
    owner: GiteaActivityUser,
    full_name: Option<String>,
    private: bool,
    html_url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GiteaActivityUser {
    login: String,
}
