use super::{Contribution, ContributionType, GitPlatform, PlatformConfig, Repository, UserInfo};
use crate::utils::http_client::create_http_client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

pub struct GitHubClient;

impl GitHubClient {
    pub fn new() -> Self {
        Self
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

        // GitHub GraphQL query for contribution calendar
        let query = r#"
            query($username: String!, $from: DateTime!, $to: DateTime!) {
                user(login: $username) {
                    contributionsCollection(from: $from, to: $to) {
                        contributionCalendar {
                            totalContributions
                            weeks {
                                contributionDays {
                                    date
                                    contributionCount
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

        let response_data: GitHubGraphQLResponse = response.json().await?;

        if let Some(errors) = response_data.errors {
            return Err(anyhow!("GitHub GraphQL errors: {:?}", errors));
        }

        let contribution_data = response_data
            .data
            .ok_or_else(|| anyhow!("No data in GitHub response"))?;

        let user_data = contribution_data
            .user
            .ok_or_else(|| anyhow!("User not found"))?;

        let calendar = user_data.contributions_collection.contribution_calendar;

        log::info!("📊 GitHub reports totalContributions: {}", calendar.total_contributions);

        // Convert GitHub contribution days to our Contribution format
        let mut contributions = Vec::new();
        for week in calendar.weeks {
            for day in week.contribution_days {
                if day.contribution_count > 0 {
                    contributions.push(Contribution {
                        date: chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")?,
                        count: day.contribution_count,
                        repository_name: None, // GitHub doesn't provide repo info in calendar
                        is_private: false,     // We'll assume public for now
                        contribution_type: ContributionType::Commit,
                    });
                }
            }
        }

        let our_total: i32 = contributions.iter().map(|c| c.count).sum();
        log::info!("📊 We collected {} contributions from {} days", our_total, contributions.len());
        if our_total != calendar.total_contributions {
            log::warn!("⚠️  Mismatch! GitHub says {} but we got {}", calendar.total_contributions, our_total);
        }

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
}

// GitHub API response types

#[derive(Debug, Deserialize)]
struct GitHubGraphQLResponse {
    data: Option<GitHubData>,
    errors: Option<Vec<GitHubGraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitHubData {
    user: Option<GitHubUserData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubUserData {
    contributions_collection: GitHubContributionsCollection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionsCollection {
    contribution_calendar: GitHubContributionCalendar,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionCalendar {
    total_contributions: i32,
    weeks: Vec<GitHubContributionWeek>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionWeek {
    contribution_days: Vec<GitHubContributionDay>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionDay {
    date: String,
    contribution_count: i32,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    email: Option<String>,
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    private: bool,
    html_url: String,
}
