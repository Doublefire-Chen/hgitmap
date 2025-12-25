use super::{Activity, ActivityType, Contribution, ContributionType, GitPlatform, PlatformConfig, Repository, UserInfo};
use crate::utils::http_client::create_http_client;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct GitHubClient;

impl GitHubClient {
    pub fn new() -> Self {
        Self
    }

    /// Fetch user profile data from GitHub
    pub async fn fetch_user_profile(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
    ) -> Result<serde_json::Value> {
        let client = create_http_client();

        log::info!("👤 Fetching GitHub profile for {}", username);

        let response = client
            .get(&format!("{}/user", config.api_base_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "hgitmap/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(anyhow!("Failed to fetch GitHub profile: status {}", status));
        }

        let profile: serde_json::Value = response.json().await?;

        log::info!("✅ Fetched profile for {}", username);

        Ok(profile)
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

    /// Fetch user's public organizations (no OAuth approval required)
    /// Note: Only returns organizations where the user has made their membership public
    pub async fn fetch_user_organizations(
        &self,
        config: &PlatformConfig,
        username: &str,
        _token: &str, // Token not needed for public orgs, but kept for API compatibility
    ) -> Result<Vec<(String, String)>> {
        let client = create_http_client();

        // Use the public /users/{username}/orgs endpoint
        // This endpoint doesn't require authentication and shows public memberships only
        let response = client
            .get(&format!("{}/users/{}/orgs", config.api_base_url, username))
            .header("User-Agent", "hgitmap/0.1.0")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch user organizations: status {}", response.status()));
        }

        let orgs: Vec<serde_json::Value> = response.json().await?;

        let mut organizations = Vec::new();

        for org in orgs {
            if let (Some(login), Some(avatar_url)) = (
                org.get("login").and_then(|v| v.as_str()),
                org.get("avatar_url").and_then(|v| v.as_str()),
            ) {
                organizations.push((login.to_string(), avatar_url.to_string()));
            }
        }

        log::info!("🏢 Fetched {} public organizations for user {}", organizations.len(), username);

        Ok(organizations)
    }

    /// Find the earliest activity date for a user in a specific organization
    /// by querying their public events and looking for the earliest event mentioning that org
    pub async fn find_earliest_org_activity_date(
        &self,
        config: &PlatformConfig,
        username: &str,
        org_login: &str,
    ) -> Result<Option<chrono::NaiveDate>> {
        let client = create_http_client();

        // Fetch user events (up to 300 events across 3 pages)
        let mut earliest_date: Option<chrono::NaiveDate> = None;
        let mut page = 1;
        let per_page = 100;

        log::info!("🔍 Searching for earliest activity in org {} for user {}", org_login, username);

        while page <= 3 {
            let response = client
                .get(&format!("{}/users/{}/events/public", config.api_base_url, username))
                .header("User-Agent", "hgitmap/0.1.0")
                .header("Accept", "application/vnd.github+json")
                .query(&[("per_page", &per_page.to_string()), ("page", &page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                log::warn!("Failed to fetch events for org date detection: status {}", response.status());
                break;
            }

            let events: Vec<serde_json::Value> = response.json().await?;

            if events.is_empty() {
                break;
            }

            // Look for events related to this organization
            for event in events {
                let mut is_org_event = false;

                // Check if this event has an org field matching our target org
                if let Some(org) = event.get("org") {
                    if let Some(org_login_field) = org.get("login").and_then(|v| v.as_str()) {
                        if org_login_field == org_login {
                            is_org_event = true;
                        }
                    }
                }

                // Also check if the event's repository belongs to this org
                // Repository names are formatted as "org/repo"
                if !is_org_event {
                    if let Some(repo) = event.get("repo") {
                        if let Some(repo_name) = repo.get("name").and_then(|v| v.as_str()) {
                            // Check if repo name starts with "org_login/"
                            if repo_name.starts_with(&format!("{}/", org_login)) {
                                is_org_event = true;
                            }
                        }
                    }
                }

                // If this is an org-related event, get its date
                if is_org_event {
                    if let Some(created_at) = event.get("created_at").and_then(|v| v.as_str()) {
                        if let Ok(event_date) = chrono::DateTime::parse_from_rfc3339(created_at) {
                            let naive_date = event_date.naive_utc().date();

                            // Update earliest_date if this is earlier
                            match earliest_date {
                                None => earliest_date = Some(naive_date),
                                Some(current_earliest) => {
                                    if naive_date < current_earliest {
                                        earliest_date = Some(naive_date);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            page += 1;
        }

        if let Some(date) = earliest_date {
            log::info!("✅ Found earliest org activity on {}", date);
        } else {
            log::info!("❌ No events found for org {}", org_login);
        }

        Ok(earliest_date)
    }

    /// Scrape organization join date from GitHub's web interface
    /// GitHub shows "Joined organization" events in the contribution activity timeline
    pub async fn scrape_org_join_date(
        &self,
        username: &str,
        org_login: &str,
    ) -> Result<Option<chrono::NaiveDate>> {
        let client = create_http_client();

        log::info!("🕷️  Scraping org join date for {} in {}", username, org_login);

        // GitHub loads the activity timeline as a fragment
        // Fetch the fragment URL directly instead of the main profile page
        let url = format!("https://github.com/{}?action=show&controller=profiles&tab=contributions&user_id={}",
            username, username);

        log::info!("📡 Fetching timeline fragment from: {}", url);

        let response = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", &format!("https://github.com/{}", username))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch GitHub timeline fragment: status {}", response.status()));
        }

        let html = response.text().await?;

        log::info!("📄 Fetched timeline HTML, length: {} bytes", html.len());

        // Debug: Save HTML to file for inspection
        if let Err(e) = std::fs::write("/tmp/github_profile_debug.html", &html) {
            log::warn!("Failed to save debug HTML: {}", e);
        } else {
            log::info!("💾 Saved HTML to /tmp/github_profile_debug.html for inspection");
        }

        // Look for: Joined the <a href="/{org_login}">...</a> organization
        // The link href will have the org login, not the display name
        let org_link_pattern = format!("<a href=\"/{}\"", org_login);

        log::info!("🔍 Searching for pattern: {}", org_link_pattern);

        // Find all occurrences of links to this org
        let mut search_pos = 0;
        let mut matches_found = 0;

        while let Some(link_idx) = html[search_pos..].find(&org_link_pattern) {
            matches_found += 1;
            let absolute_link_idx = search_pos + link_idx;

            log::info!("📌 Found link #{} at position {}", matches_found, absolute_link_idx);

            // Look backwards from the link to see if "Joined the" appears nearby (within 100 chars)
            let search_start = absolute_link_idx.saturating_sub(100);
            let before_link = &html[search_start..absolute_link_idx];

            if before_link.contains("Joined the") {
                log::info!("✓ Found 'Joined the' before link #{}", matches_found);

                // Found a "Joined the org" event! Now look for the date
                // The structure is: Joined the <a href="/ORG">...</a> organization</h4>
                //                   <a ... href="/ORG"><time>on Dec 14</time></a>
                // Look forward from the link for <time> tag (within 1000 chars to account for whitespace and nested tags)
                let search_end = (absolute_link_idx + 1000).min(html.len());
                let after_link = &html[absolute_link_idx..search_end];

                // Look for <time> tag (it might be nested in another <a> tag)
                if let Some(time_start) = after_link.find("<time>") {
                    log::info!("✓ Found <time> tag at offset {}", time_start);
                    let content_start = time_start + 6; // Length of "<time>"
                    if let Some(time_end) = after_link[content_start..].find("</time>") {
                        let time_content = &after_link[content_start..content_start + time_end];

                        log::info!("📅 Time content: '{}'", time_content);

                        // Parse date like "on Dec 14" or "Dec 14"
                        let date_str = time_content.trim().trim_start_matches("on").trim();

                        log::info!("📅 Parsed date string: '{}'", date_str);

                        // Parse date (format: "Dec 14", "Jan 5", etc.)
                        if let Some(parsed_date) = self.parse_github_date(date_str) {
                            log::info!("✅ Scraped org join date: {}", parsed_date);
                            return Ok(Some(parsed_date));
                        } else {
                            log::warn!("⚠️  Failed to parse date: '{}'", date_str);
                        }
                    }
                } else {
                    log::info!("✗ No <time> tag found after link #{}", matches_found);
                    // Debug: show what's actually there
                    let preview_len = 200.min(after_link.len());
                    log::info!("📝 Content after link: {}", &after_link[..preview_len]);
                }

                // Also try looking backwards for a <time datetime="..."> tag
                let before_section = &html[search_start..absolute_link_idx + 200];
                if let Some(time_start) = before_section.rfind("<time datetime=\"") {
                    let datetime_start = time_start + 16;
                    if let Some(datetime_end) = before_section[datetime_start..].find("\"") {
                        let datetime_str = &before_section[datetime_start..datetime_start + datetime_end];

                        if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(datetime_str) {
                            let join_date = parsed_date.naive_utc().date();
                            log::info!("✅ Scraped org join date: {}", join_date);
                            return Ok(Some(join_date));
                        } else if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
                            log::info!("✅ Scraped org join date: {}", naive_date);
                            return Ok(Some(naive_date));
                        }
                    }
                }
            } else {
                log::info!("✗ No 'Joined the' found before link #{}", matches_found);
            }

            search_pos = absolute_link_idx + 1;
        }

        log::warn!("⚠️  Could not find org join date in scraped HTML (found {} link occurrences)", matches_found);
        Ok(None)
    }

    /// Parse GitHub's abbreviated date format (e.g., "Dec 14", "Jan 5")
    /// Returns the date assuming it's from this year or last year
    fn parse_github_date(&self, date_str: &str) -> Option<chrono::NaiveDate> {
        // Date format: "Dec 14", "Jan 5", etc.
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }

        let month_str = parts[0];
        let day_str = parts[1];

        let day: u32 = day_str.parse().ok()?;

        // Map month abbreviations to numbers
        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return None,
        };

        // Determine the year - assume current year, but if the date is in the future, use last year
        let now = chrono::Utc::now();
        let current_year = now.year();

        // Try current year first
        if let Some(date) = chrono::NaiveDate::from_ymd_opt(current_year, month, day) {
            // If this date is in the future, it must be from last year
            if date > now.naive_utc().date() {
                return chrono::NaiveDate::from_ymd_opt(current_year - 1, month, day);
            }
            return Some(date);
        }

        None
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

    /// Search for commits on specific dates using GitHub REST API
    /// This is used as a fallback when commitContributionsByRepository doesn't return data
    async fn search_commits_for_dates(
        &self,
        config: &PlatformConfig,
        username: &str,
        token: &str,
        dates: &[chrono::NaiveDate],
    ) -> Result<HashMap<chrono::NaiveDate, Vec<String>>> {
        let client = create_http_client();
        let mut date_repos: HashMap<chrono::NaiveDate, Vec<String>> = HashMap::new();

        log::info!("🔍 Searching for commits on {} dates with missing repository info", dates.len());

        // Group consecutive dates to minimize API calls
        let mut date_ranges: Vec<(chrono::NaiveDate, chrono::NaiveDate)> = Vec::new();
        let mut sorted_dates = dates.to_vec();
        sorted_dates.sort();

        for date in &sorted_dates {
            if let Some(last_range) = date_ranges.last_mut() {
                // If this date is consecutive to the last range, extend it
                if *date == last_range.1 + chrono::Duration::days(1) {
                    last_range.1 = *date;
                    continue;
                }
            }
            // Start a new range
            date_ranges.push((*date, *date));
        }

        log::info!("📅 Grouped {} dates into {} ranges for search", dates.len(), date_ranges.len());

        // Search each date range using REST API
        for (from_date, to_date) in date_ranges {
            let from_str = from_date.format("%Y-%m-%d").to_string();
            let to_str = to_date.format("%Y-%m-%d").to_string();

            // GitHub REST API search for commits
            // https://docs.github.com/en/rest/search/search#search-commits
            let search_query = format!("author:{} committer-date:{}..{}", username, from_str, to_str);

            log::debug!("🔎 REST API Search: {}", search_query);

            let mut page = 1;
            let per_page = 100;

            loop {
                let url = format!(
                    "{}/search/commits?q={}&per_page={}&page={}",
                    config.api_base_url,
                    urlencoding::encode(&search_query),
                    per_page,
                    page
                );

                let response = client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("User-Agent", "hgitmap/0.1.0")
                    .header("Accept", "application/vnd.github.cloak-preview+json") // Required for commit search
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error".to_string());
                    log::warn!("REST API search failed: status {} - Error: {}", status, error_body);
                    break;
                }

                let search_result: serde_json::Value = response.json().await?;

                let items = search_result
                    .get("items")
                    .and_then(|v| v.as_array());

                if let Some(commits) = items {
                    if commits.is_empty() {
                        break;
                    }

                    for commit in commits {
                        if let (Some(commit_obj), Some(repo)) = (
                            commit.get("commit"),
                            commit.get("repository"),
                        ) {
                            // Get commit date
                            if let Some(committer_date_str) = commit_obj
                                .get("committer")
                                .and_then(|c| c.get("date"))
                                .and_then(|d| d.as_str())
                            {
                                if let Ok(commit_date) = chrono::DateTime::parse_from_rfc3339(committer_date_str) {
                                    let date = commit_date.naive_utc().date();

                                    // Get repository name
                                    if let Some(repo_full_name) = repo.get("full_name").and_then(|n| n.as_str()) {
                                        let entry = date_repos.entry(date).or_insert_with(Vec::new);
                                        if !entry.contains(&repo_full_name.to_string()) {
                                            entry.push(repo_full_name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    log::debug!("📄 Page {}: Found {} commits", page, commits.len());

                    // Check if there are more pages
                    let total_count = search_result
                        .get("total_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    if (page * per_page) as i64 >= total_count || page >= 10 {
                        break;
                    }

                    page += 1;
                } else {
                    log::debug!("No commits found in search response");
                    break;
                }

                // Be nice to the API - small delay between pages
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Be nice to the API - delay between searches
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        log::info!("✅ Search found repository names for {} dates", date_repos.len());

        Ok(date_repos)
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

        // First, fetch the calendar for accurate total counts
        let calendar_query = r#"
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
                "query": calendar_query,
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

        let calendar_response: GitHubContributionCalendarOnlyResponse = response.json().await?;

        if let Some(errors) = calendar_response.errors {
            return Err(anyhow!("GitHub GraphQL errors: {:?}", errors));
        }

        let calendar_data = calendar_response
            .data
            .ok_or_else(|| anyhow!("No data in GitHub response"))?;

        let calendar_user_data = calendar_data
            .user
            .ok_or_else(|| anyhow!("User not found"))?;

        let calendar = calendar_user_data.contributions_collection.contribution_calendar;

        log::info!("📊 GitHub contributionCalendar reports {} total contributions", calendar.total_contributions);

        // Now fetch repository data with pagination to get complete privacy info
        use std::collections::HashMap;
        let mut date_privacy_map: HashMap<chrono::NaiveDate, (bool, Vec<(String, i32)>)> = HashMap::new();

        let mut cursor: Option<String> = None;
        let mut page_num = 0;
        let max_pages = 25; // Increased from 10 to 25 (2500 contribution days total)

        loop {
            page_num += 1;

            let repo_query = if let Some(ref after_cursor) = cursor {
                format!(r#"
                    query($username: String!, $from: DateTime!, $to: DateTime!) {{
                        user(login: $username) {{
                            contributionsCollection(from: $from, to: $to) {{
                                commitContributionsByRepository(maxRepositories: 100) {{
                                    repository {{
                                        nameWithOwner
                                        isPrivate
                                    }}
                                    contributions(first: 100, after: "{}") {{
                                        pageInfo {{
                                            hasNextPage
                                            endCursor
                                        }}
                                        nodes {{
                                            occurredAt
                                            commitCount
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                "#, after_cursor)
            } else {
                r#"
                    query($username: String!, $from: DateTime!, $to: DateTime!) {
                        user(login: $username) {
                            contributionsCollection(from: $from, to: $to) {
                                commitContributionsByRepository(maxRepositories: 100) {
                                    repository {
                                        nameWithOwner
                                        isPrivate
                                    }
                                    contributions(first: 100) {
                                        pageInfo {
                                            hasNextPage
                                            endCursor
                                        }
                                        nodes {
                                            occurredAt
                                            commitCount
                                        }
                                    }
                                }
                            }
                        }
                    }
                "#.to_string()
            };

            let repo_response = client
                .post(&format!("{}/graphql", config.api_base_url))
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "hgitmap/0.1.0")
                .json(&json!({
                    "query": repo_query,
                    "variables": variables,
                }))
                .send()
                .await?;

            if !repo_response.status().is_success() {
                log::warn!("Failed to fetch repository data page {}: status {}", page_num, repo_response.status());
                break;
            }

            let repo_data: GitHubRepoContributionResponse = repo_response.json().await?;

            if let Some(errors) = repo_data.errors {
                log::warn!("GraphQL errors on page {}: {:?}", page_num, errors);
                break;
            }

            let Some(data) = repo_data.data else {
                log::warn!("No data in page {}", page_num);
                break;
            };

            let Some(user_data) = data.user else {
                log::warn!("No user in page {}", page_num);
                break;
            };

            let repos_data = user_data.contributions_collection.commit_contributions_by_repository;
            let mut has_next_page = false;
            let mut next_cursor: Option<String> = None;

            log::info!("📄 Processing page {} with {} repositories", page_num, repos_data.len());

            for repo_contribution in repos_data {
                let repo_name = repo_contribution.repository.name_with_owner;
                let is_private = repo_contribution.repository.is_private;

                // Check if there are more pages for this repository
                if let Some(ref page_info) = repo_contribution.contributions.page_info {
                    if page_info.has_next_page {
                        has_next_page = true;
                        next_cursor = page_info.end_cursor.clone();
                    }
                }

                for node in repo_contribution.contributions.nodes {
                    if node.commit_count > 0 {
                        let occurred_at = chrono::DateTime::parse_from_rfc3339(&node.occurred_at)
                            .map_err(|e| anyhow!("Failed to parse date: {}", e))?;
                        let date = occurred_at.naive_utc().date();

                        let entry = date_privacy_map.entry(date).or_insert((false, Vec::new()));
                        // If ANY repo on this date is private, mark the whole day as private
                        if is_private {
                            entry.0 = true;
                        }
                        // Store repo name WITH commit count
                        entry.1.push((repo_name.clone(), node.commit_count));
                    }
                }
            }

            // Stop if no more pages or reached max pages
            if !has_next_page || page_num >= max_pages {
                log::info!("📊 Finished fetching repository data after {} pages", page_num);
                break;
            }

            cursor = next_cursor;
        }

        log::info!("📊 Mapped privacy info for {} days from {} pages of repository data",
            date_privacy_map.len(), page_num);

        // Convert calendar data to our Contribution format, enriched with privacy info
        let mut contributions = Vec::new();
        let mut dates_without_repo: Vec<chrono::NaiveDate> = Vec::new();

        for week in calendar.weeks {
            for day in week.contribution_days {
                if day.contribution_count > 0 {
                    let date = chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")
                        .map_err(|e| anyhow!("Failed to parse date: {}", e))?;

                    // Check if we have repo data for this date
                    if let Some((is_private, repos_with_counts)) = date_privacy_map.get(&date) {
                        // We have repository data for this date
                        // Use the calendar's total count as source of truth, but attribute to repositories

                        // Calculate how many commits we tracked via repos
                        let tracked_commits: i32 = repos_with_counts.iter().map(|(_, count)| count).sum();

                        // Calendar count includes commits + PRs + issues + reviews
                        // If calendar count > tracked commits, there are non-commit contributions
                        let non_commit_contributions = day.contribution_count.saturating_sub(tracked_commits);

                        // Create separate contribution for each repository
                        for (repo_name, commit_count) in repos_with_counts {
                            contributions.push(Contribution {
                                date,
                                count: *commit_count,
                                repository_name: Some(repo_name.clone()),
                                is_private: *is_private,
                                contribution_type: ContributionType::Commit,
                            });
                        }

                        // If there are non-commit contributions, add them as a separate entry
                        // This ensures the total matches GitHub's calendar
                        if non_commit_contributions > 0 {
                            log::debug!("📊 Date {}: calendar shows {} total, tracked {} commits, adding {} non-commit contributions",
                                date, day.contribution_count, tracked_commits, non_commit_contributions);
                            // Use NULL for repository since these are non-commit contributions (PRs, issues, reviews)
                            // This avoids violating the unique constraint
                            contributions.push(Contribution {
                                date,
                                count: non_commit_contributions,
                                repository_name: None,
                                is_private: *is_private,
                                contribution_type: ContributionType::Commit, // Mixed type
                            });
                        }
                    } else {
                        // No repo info - create one contribution with NULL
                        contributions.push(Contribution {
                            date,
                            count: day.contribution_count,
                            repository_name: None,
                            is_private: false,
                            contribution_type: ContributionType::Commit,
                        });
                        dates_without_repo.push(date);
                    }
                }
            }
        }

        // Use Search API as fallback for dates without repository info
        if !dates_without_repo.is_empty() {
            log::info!("🔍 {} dates have contributions but no repository info, using Search API fallback", dates_without_repo.len());

            match self.search_commits_for_dates(config, username, token, &dates_without_repo).await {
                Ok(search_results) => {
                    // Update contributions with search results
                    // For dates with multiple repos found, we can't split the count accurately,
                    // so we'll create separate contributions with the full count for now
                    let mut updated_count = 0;
                    for contrib in &mut contributions {
                        if contrib.repository_name.is_none() {
                            if let Some(repos) = search_results.get(&contrib.date) {
                                if !repos.is_empty() {
                                    // Found repo(s) for this date
                                    // Use the first repo found (best guess)
                                    contrib.repository_name = Some(repos[0].clone());
                                    updated_count += 1;
                                }
                            }
                        }
                    }
                    log::info!("✅ Search API found repository names for {} / {} dates", updated_count, dates_without_repo.len());
                }
                Err(e) => {
                    log::warn!("⚠️  Search API fallback failed: {}", e);
                }
            }
        }

        let total: i32 = contributions.iter().map(|c| c.count).sum();
        let with_repos = contributions.iter().filter(|c| c.repository_name.is_some()).count();
        let without_repos = contributions.len() - with_repos;

        log::info!("📊 Collected {} contributions across {} days (calendar total: {})",
            total, contributions.len(), calendar.total_contributions);
        log::info!("📊 Repository attribution: {} with repos, {} without", with_repos, without_repos);

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

// Calendar-only response (first request)
#[derive(Debug, Deserialize)]
struct GitHubContributionCalendarOnlyResponse {
    data: Option<GitHubContributionCalendarOnlyData>,
    errors: Option<Vec<GitHubGraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GitHubContributionCalendarOnlyData {
    user: Option<GitHubContributionCalendarOnlyUserData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionCalendarOnlyUserData {
    contributions_collection: GitHubContributionCalendarOnlyCollection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionCalendarOnlyCollection {
    contribution_calendar: GitHubContributionCalendar,
}

// Repository contribution response (paginated requests)
#[derive(Debug, Deserialize)]
struct GitHubRepoContributionResponse {
    data: Option<GitHubRepoContributionData>,
    errors: Option<Vec<GitHubGraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoContributionData {
    user: Option<GitHubRepoContributionUserData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubRepoContributionUserData {
    contributions_collection: GitHubRepoContributionCollection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubRepoContributionCollection {
    commit_contributions_by_repository: Vec<GitHubRepoContribution>,
}

// Shared structs
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
#[serde(rename_all = "camelCase")]
struct GitHubRepoContribution {
    repository: GitHubRepositoryInfo,
    contributions: GitHubContributionNodesWithPageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubRepositoryInfo {
    name_with_owner: String,
    is_private: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionNodesWithPageInfo {
    page_info: Option<GitHubPageInfo>,
    nodes: Vec<GitHubContributionNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubPageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubContributionNode {
    occurred_at: String,
    commit_count: i32,
}

#[derive(Debug, Deserialize)]
struct GitHubGraphQLError {
    #[allow(dead_code)]
    message: String,
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
