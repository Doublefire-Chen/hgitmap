use super::{Activity, ActivityType, Contribution, ContributionType, GitPlatform, PlatformConfig, Repository, UserInfo};
use crate::utils::http_client::create_http_client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct GitHubClient;

impl GitHubClient {
    pub fn new() -> Self {
        Self
    }

    /// Fetch repository details including primary language
    async fn fetch_repository_details(
        &self,
        config: &PlatformConfig,
        token: &str,
        repo_name: &str,
    ) -> Result<serde_json::Value> {
        let client = create_http_client();

        let response = client
            .get(&format!("{}/repos/{}", config.api_base_url, repo_name))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch repository details"));
        }

        let repo_data: serde_json::Value = response.json().await?;
        Ok(repo_data)
    }

    /// Fetch repository creation activities using GraphQL (no time limit)
    async fn fetch_repository_creation_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        let client = create_http_client();

        // GraphQL query to get all repositories with creation dates
        let query = r#"
            query($username: String!) {
                user(login: $username) {
                    repositories(first: 100, orderBy: {field: CREATED_AT, direction: DESC}, ownerAffiliations: OWNER) {
                        nodes {
                            nameWithOwner
                            createdAt
                            description
                            isPrivate
                            primaryLanguage {
                                name
                            }
                            url
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "username": username,
        });

        let response = client
            .post(&format!("{}/graphql", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .json(&json!({
                "query": query,
                "variables": variables,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!(
                "GitHub API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_data: serde_json::Value = response.json().await?;

        if let Some(errors) = response_data.get("errors") {
            return Err(anyhow!("GitHub GraphQL errors: {:?}", errors));
        }

        let repos = response_data
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("repositories"))
            .and_then(|r| r.get("nodes"))
            .and_then(|n| n.as_array())
            .ok_or_else(|| anyhow!("Invalid response structure"))?;

        let mut activities = Vec::new();

        for repo in repos {
            let created_at_str = repo.get("createdAt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing createdAt"))?;

            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
            let created_at_utc = created_at.with_timezone(&chrono::Utc);

            // Filter by date range
            if created_at_utc < from || created_at_utc > to {
                continue;
            }

            let name_with_owner = repo.get("nameWithOwner")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing nameWithOwner"))?;

            let description = repo.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let is_private = repo.get("isPrivate")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let primary_language = repo.get("primaryLanguage")
                .and_then(|l| l.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            let url = repo.get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            activities.push(Activity {
                activity_type: ActivityType::RepositoryCreated,
                date: created_at_utc.naive_utc().date(),
                metadata: json!({
                    "name": name_with_owner,
                    "description": description,
                    "created_at": created_at_str,
                }),
                repository_name: Some(name_with_owner.to_string()),
                repository_url: url,
                is_private,
                count: 1,
                primary_language,
                organization_name: None,
                organization_avatar_url: None,
            });
        }

        log::info!("📦 Fetched {} repository creation activities from GraphQL", activities.len());

        Ok(activities)
    }

    /// Fetch PR and issue activities using GraphQL search (no time limit)
    pub async fn fetch_pr_and_issue_activities(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Activity>> {
        let client = create_http_client();
        let mut all_activities = Vec::new();

        // Format dates for GitHub search query (YYYY-MM-DD)
        let from_str = from.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        // Fetch pull requests
        let pr_query = r#"
            query($searchQuery: String!) {
                search(query: $searchQuery, type: ISSUE, first: 100) {
                    nodes {
                        ... on PullRequest {
                            title
                            number
                            state
                            createdAt
                            url
                            body
                            comments {
                                totalCount
                            }
                            repository {
                                nameWithOwner
                                isPrivate
                            }
                        }
                    }
                }
            }
        "#;

        let pr_search_query = format!("author:{} type:pr created:{}..{}", username, from_str, to_str);

        let pr_response = client
            .post(&format!("{}/graphql", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .json(&json!({
                "query": pr_query,
                "variables": json!({
                    "searchQuery": pr_search_query,
                }),
            }))
            .send()
            .await?;

        if pr_response.status().is_success() {
            let pr_data: serde_json::Value = pr_response.json().await?;

            if let Some(nodes) = pr_data.get("data")
                .and_then(|d| d.get("search"))
                .and_then(|s| s.get("nodes"))
                .and_then(|n| n.as_array())
            {
                for node in nodes {
                    if let (Some(title), Some(number), Some(state), Some(created_at), Some(url), Some(repo)) = (
                        node.get("title").and_then(|v| v.as_str()),
                        node.get("number").and_then(|v| v.as_i64()),
                        node.get("state").and_then(|v| v.as_str()),
                        node.get("createdAt").and_then(|v| v.as_str()),
                        node.get("url").and_then(|v| v.as_str()),
                        node.get("repository"),
                    ) {
                        let created_at_parsed = chrono::DateTime::parse_from_rfc3339(created_at)
                            .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
                        let date = created_at_parsed.naive_utc().date();

                        let repo_name = repo.get("nameWithOwner")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let is_private = repo.get("isPrivate")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        // Extract body (description) and comment count
                        let body = node.get("body")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let comment_count = node.get("comments")
                            .and_then(|c| c.get("totalCount"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        let mut metadata = json!({
                            "title": title,
                            "number": number,
                            "state": state,
                            "repository": repo_name,
                            "url": url,
                            "comment_count": comment_count,
                        });

                        // Only include body if it exists and is not empty
                        if let Some(body_text) = body {
                            if !body_text.trim().is_empty() {
                                metadata.as_object_mut().unwrap().insert("body".to_string(), json!(body_text));
                            }
                        }

                        all_activities.push(Activity {
                            activity_type: ActivityType::PullRequest,
                            date,
                            metadata,
                            repository_name: Some(repo_name.to_string()),
                            repository_url: Some(format!("https://github.com/{}", repo_name)),
                            is_private,
                            count: 1,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                }
            }
        }

        // Fetch issues (similar query but for issues)
        let issue_query = r#"
            query($searchQuery: String!) {
                search(query: $searchQuery, type: ISSUE, first: 100) {
                    nodes {
                        ... on Issue {
                            title
                            number
                            state
                            createdAt
                            url
                            body
                            comments {
                                totalCount
                            }
                            repository {
                                nameWithOwner
                                isPrivate
                            }
                        }
                    }
                }
            }
        "#;

        let issue_search_query = format!("author:{} type:issue created:{}..{}", username, from_str, to_str);

        let issue_response = client
            .post(&format!("{}/graphql", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .json(&json!({
                "query": issue_query,
                "variables": json!({
                    "searchQuery": issue_search_query,
                }),
            }))
            .send()
            .await?;

        if issue_response.status().is_success() {
            let issue_data: serde_json::Value = issue_response.json().await?;

            if let Some(nodes) = issue_data.get("data")
                .and_then(|d| d.get("search"))
                .and_then(|s| s.get("nodes"))
                .and_then(|n| n.as_array())
            {
                for node in nodes {
                    if let (Some(title), Some(number), Some(state), Some(created_at), Some(url), Some(repo)) = (
                        node.get("title").and_then(|v| v.as_str()),
                        node.get("number").and_then(|v| v.as_i64()),
                        node.get("state").and_then(|v| v.as_str()),
                        node.get("createdAt").and_then(|v| v.as_str()),
                        node.get("url").and_then(|v| v.as_str()),
                        node.get("repository"),
                    ) {
                        let created_at_parsed = chrono::DateTime::parse_from_rfc3339(created_at)
                            .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
                        let date = created_at_parsed.naive_utc().date();

                        let repo_name = repo.get("nameWithOwner")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let is_private = repo.get("isPrivate")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        // Extract body (description) and comment count
                        let body = node.get("body")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let comment_count = node.get("comments")
                            .and_then(|c| c.get("totalCount"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        let mut metadata = json!({
                            "title": title,
                            "number": number,
                            "state": state,
                            "repository": repo_name,
                            "url": url,
                            "comment_count": comment_count,
                        });

                        // Only include body if it exists and is not empty
                        if let Some(body_text) = body {
                            if !body_text.trim().is_empty() {
                                metadata.as_object_mut().unwrap().insert("body".to_string(), json!(body_text));
                            }
                        }

                        all_activities.push(Activity {
                            activity_type: ActivityType::Issue,
                            date,
                            metadata,
                            repository_name: Some(repo_name.to_string()),
                            repository_url: Some(format!("https://github.com/{}", repo_name)),
                            is_private,
                            count: 1,
                            primary_language: None,
                            organization_name: None,
                            organization_avatar_url: None,
                        });
                    }
                }
            }
        }

        log::info!("🔍 Fetched {} PR/issue activities from GraphQL search", all_activities.len());

        Ok(all_activities)
    }
}

#[async_trait]
impl GitPlatform for GitHubClient {
    async fn fetch_contributions(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Contribution>> {
        let client = create_http_client();

        // GitHub GraphQL query for contributions by repository with per-day details
        // Note: GitHub's API has a hard limit of 100 nodes for contributions connection
        let query = r#"
            query($username: String!, $from: DateTime!, $to: DateTime!) {
                user(login: $username) {
                    contributionsCollection(from: $from, to: $to) {
                        commitContributionsByRepository(maxRepositories: 100) {
                            repository {
                                nameWithOwner
                                isPrivate
                            }
                            contributions(first: 100) {
                                nodes {
                                    occurredAt
                                    commitCount
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "username": username,
            "from": from.to_rfc3339(),
            "to": to.to_rfc3339(),
        });

        let response = client
            .post(&format!("{}/graphql", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .json(&json!({
                "query": query,
                "variables": variables,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!(
                "GitHub API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_data: GitHubContributionsByRepoResponse = response.json().await?;

        if let Some(errors) = response_data.errors {
            return Err(anyhow!("GitHub GraphQL errors: {:?}", errors));
        }

        let contribution_data = response_data
            .data
            .ok_or_else(|| anyhow!("No data in GitHub response"))?;

        let user_data = contribution_data
            .user
            .ok_or_else(|| anyhow!("User not found"))?;

        let repos_data = user_data.contributions_collection.commit_contributions_by_repository;

        log::info!("📊 GitHub returned contributions for {} repositories", repos_data.len());

        // Convert to our Contribution format
        let mut contributions = Vec::new();
        for repo_contribution in repos_data {
            let repo_name = repo_contribution.repository.name_with_owner;
            let is_private = repo_contribution.repository.is_private;

            for node in repo_contribution.contributions.nodes {
                if node.commit_count > 0 {
                    // Parse the occurredAt datetime and extract just the date
                    let occurred_at = chrono::DateTime::parse_from_rfc3339(&node.occurred_at)
                        .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
                    let date = occurred_at.naive_utc().date();

                    contributions.push(Contribution {
                        date,
                        count: node.commit_count,
                        repository_name: Some(repo_name.clone()),
                        is_private,
                        contribution_type: ContributionType::Commit,
                    });
                }
            }
        }

        let total: i32 = contributions.iter().map(|c| c.count).sum();
        log::info!("📊 Collected {} contributions from {} days across repositories", total, contributions.len());

        Ok(contributions)
    }

    async fn validate_token(&self, config: &PlatformConfig, token: &str) -> Result<UserInfo> {
        let client = create_http_client();

        // Use GitHub's user endpoint to validate token and get user info
        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Invalid GitHub token: status {}", status));
        }

        let user: GitHubUser = response.json().await?;

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

        let response = client
            .get(&format!("{}/user/repos", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .query(&[("per_page", "100"), ("affiliation", "owner,collaborator")])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Failed to fetch repositories: status {}", status));
        }

        let repos: Vec<GitHubRepo> = response.json().await?;

        Ok(repos
            .into_iter()
            .map(|repo| Repository {
                name: repo.name,
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

        // Fetch user events from GitHub API
        let mut all_events = Vec::new();
        let mut page = 1;
        let per_page = 100;

        // GitHub API only allows fetching up to 300 events
        while page <= 3 {
            let response = client
                .get(&format!("{}/users/{}/events", config.api_base_url, username))
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "hgitmap/0.1.0")
                .query(&[("per_page", &per_page.to_string()), ("page", &page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                return Err(anyhow!("Failed to fetch GitHub events: status {}", status));
            }

            let events: Vec<GitHubEvent> = response.json().await?;

            log::info!("📥 Fetched {} events from GitHub (page {})", events.len(), page);

            if events.is_empty() {
                break;
            }

            all_events.extend(events);
            page += 1;
        }

        log::info!("📊 Total events fetched from GitHub: {}", all_events.len());

        // Group events by date and type
        let mut activities = Vec::new();
        let mut commit_groups: HashMap<String, Vec<GitHubEvent>> = HashMap::new();
        let mut filtered_count = 0;

        log::info!("🔍 Filtering events from {} to {}", from, to);

        for event in all_events {
            // Parse event date
            let event_date = chrono::DateTime::parse_from_rfc3339(&event.created_at)
                .map_err(|e| anyhow!("Failed to parse date: {}", e))?;

            // Convert to UTC for comparison
            let event_date_utc = event_date.with_timezone(&chrono::Utc);

            // Filter by date range
            if event_date_utc < from || event_date_utc > to {
                filtered_count += 1;
                continue;
            }

            let naive_date = event_date.naive_utc().date();

            match event.event_type.as_str() {
                "PushEvent" => {
                    // Debug: log the payload to see what GitHub actually returns
                    if let Some(ref payload) = event.payload {
                        log::info!("🔍 PushEvent payload for {}: {}", event.repo.name, serde_json::to_string_pretty(payload).unwrap_or_else(|_| "failed to serialize".to_string()));
                        log::info!("🔍 Payload keys: {:?}", payload.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                    } else {
                        log::warn!("⚠️  PushEvent has NO payload for {}", event.repo.name);
                    }
                    // Group commits by date and repository
                    let key = format!("{}_{}", naive_date, event.repo.name);
                    commit_groups.entry(key).or_insert_with(Vec::new).push(event);
                }
                "CreateEvent" => {
                    if let Some(ref payload) = event.payload {
                        if payload.get("ref_type").and_then(|v| v.as_str()) == Some("repository") {
                            // Fetch repository details to get primary language
                            let repo_details = self.fetch_repository_details(
                                config,
                                token,
                                &event.repo.name
                            ).await.ok();

                            let primary_language = repo_details.as_ref()
                                .and_then(|r| r.get("language"))
                                .and_then(|l| l.as_str())
                                .map(|s| s.to_string());

                            activities.push(Activity {
                                activity_type: ActivityType::RepositoryCreated,
                                date: naive_date,
                                metadata: json!({
                                    "name": event.repo.name,
                                    "description": payload.get("description"),
                                    "created_at": event.created_at,
                                }),
                                repository_name: Some(event.repo.name.clone()),
                                repository_url: Some(format!("https://github.com/{}", event.repo.name)),
                                is_private: false, // GitHub events API doesn't expose privacy
                                count: 1,
                                primary_language,
                                organization_name: None,
                                organization_avatar_url: None,
                            });
                        }
                    }
                }
                "PullRequestEvent" => {
                    if let Some(ref payload) = event.payload {
                        if let Some(pr) = payload.get("pull_request") {
                            activities.push(Activity {
                                activity_type: ActivityType::PullRequest,
                                date: naive_date,
                                metadata: json!({
                                    "title": pr.get("title"),
                                    "number": pr.get("number"),
                                    "state": pr.get("state"),
                                    "repository": event.repo.name,
                                    "url": pr.get("html_url"),
                                }),
                                repository_name: Some(event.repo.name.clone()),
                                repository_url: Some(format!("https://github.com/{}", event.repo.name)),
                                is_private: false,
                                count: 1,
                                primary_language: None,
                                organization_name: None,
                                organization_avatar_url: None,
                            });
                        }
                    }
                }
                "IssuesEvent" => {
                    if let Some(ref payload) = event.payload {
                        if let Some(issue) = payload.get("issue") {
                            activities.push(Activity {
                                activity_type: ActivityType::Issue,
                                date: naive_date,
                                metadata: json!({
                                    "title": issue.get("title"),
                                    "number": issue.get("number"),
                                    "state": issue.get("state"),
                                    "repository": event.repo.name,
                                    "url": issue.get("html_url"),
                                }),
                                repository_name: Some(event.repo.name.clone()),
                                repository_url: Some(format!("https://github.com/{}", event.repo.name)),
                                is_private: false,
                                count: 1,
                                primary_language: None,
                                organization_name: None,
                                organization_avatar_url: None,
                            });
                        }
                    }
                }
                "ForkEvent" => {
                    activities.push(Activity {
                        activity_type: ActivityType::Fork,
                        date: naive_date,
                        metadata: json!({
                            "repository": event.repo.name,
                        }),
                        repository_name: Some(event.repo.name.clone()),
                        repository_url: Some(format!("https://github.com/{}", event.repo.name)),
                        is_private: false,
                        count: 1,
                        primary_language: None,
                        organization_name: None,
                        organization_avatar_url: None,
                    });
                }
                "MemberEvent" => {
                    // Organization joined might be in OrgEvent, but MemberEvent can indicate collaboration
                    if let Some(ref org) = event.org {
                        activities.push(Activity {
                            activity_type: ActivityType::OrganizationJoined,
                            date: naive_date,
                            metadata: json!({
                                "organization": org.login,
                                "avatar_url": org.avatar_url,
                                "joined_at": event.created_at,
                            }),
                            repository_name: None,
                            repository_url: None,
                            is_private: false,
                            count: 1,
                            primary_language: None,
                            organization_name: Some(org.login.clone()),
                            organization_avatar_url: org.avatar_url.clone(),
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

            log::info!("📦 Processing {} commit events for repo: {}", events.len(), repo_name);

            // Since GitHub Events API payload doesn't include commit count in the structure,
            // we'll count the number of PushEvents as an approximation.
            // Each PushEvent represents at least one push activity.
            let total_commits: i32 = events.len() as i32;

            log::info!("📊 Total push events for {}: {}", repo_name, total_commits);

            let repositories = vec![json!({
                "name": repo_name,
                "commit_count": total_commits,
            })];

            activities.push(Activity {
                activity_type: ActivityType::Commit,
                date: naive_date,
                metadata: json!({
                    "repositories": repositories,
                    "total_count": total_commits,
                }),
                repository_name: Some(repo_name.clone()),
                repository_url: Some(format!("https://github.com/{}", repo_name)),
                is_private: false,
                count: total_commits,
                primary_language: None,
                organization_name: None,
                organization_avatar_url: None,
            });
        }

        log::info!(
            "📊 Activity processing complete: {} events filtered out, {} activities created",
            filtered_count,
            activities.len()
        );

        // Sort by date descending
        activities.sort_by(|a, b| b.date.cmp(&a.date));

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
        // Use the helper method defined in impl GitHubClient
        GitHubClient::fetch_repository_creation_activities(self, config, username, token, from, to).await
    }
}

// GitHub API response types

// New response types for commitContributionsByRepository query
#[derive(Debug, Deserialize)]
struct GitHubContributionsByRepoResponse {
    data: Option<GitHubContributionsByRepoData>,
    errors: Option<Vec<GitHubGraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphQLError {
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitHubContributionsByRepoData {
    user: Option<GitHubContributionsByRepoUserData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionsByRepoUserData {
    contributions_collection: GitHubContributionsByRepoCollection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionsByRepoCollection {
    commit_contributions_by_repository: Vec<GitHubRepoContribution>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubRepoContribution {
    repository: GitHubRepositoryInfo,
    contributions: GitHubContributionNodes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubRepositoryInfo {
    name_with_owner: String,
    is_private: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubContributionNodes {
    nodes: Vec<GitHubContributionNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionNode {
    occurred_at: String,
    commit_count: i32,
}


#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    email: Option<String>,
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRepo {
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
}

// GitHub Events API response types

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubEvent {
    #[serde(rename = "type")]
    event_type: String,
    created_at: String,
    repo: GitHubEventRepo,
    payload: Option<serde_json::Value>, // Use raw JSON to see all fields
    org: Option<GitHubEventOrg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubEventRepo {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubEventOrg {
    login: String,
    avatar_url: Option<String>,
}
