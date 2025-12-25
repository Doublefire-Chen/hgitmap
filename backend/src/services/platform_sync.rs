use anyhow::{Context, Result};
use chrono::{Datelike, Utc};
use sea_orm::*;
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::json;

use crate::models::{contribution, git_platform_account, heatmap_theme, activity};
use crate::services::heatmap_generator::HeatmapGenerator;
use crate::services::git_platforms::{github::GitHubClient, gitea::GiteaClient, GitPlatform, PlatformConfig, Contribution, Activity, ActivityType};
use crate::utils::{config::Config, encryption};

pub struct PlatformSyncService {
    db: DatabaseConnection,
    config: Config,
}

impl PlatformSyncService {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        Self { db, config }
    }

    /// Sync all active platform accounts for a user (current year only)
    pub async fn sync_user_data(&self, user_id: Uuid) -> Result<SyncResult> {
        log::info!("Starting sync for user: {}", user_id);

        let mut result = SyncResult {
            user_id,
            platforms_synced: 0,
            contributions_added: 0,
            contributions_updated: 0,
            errors: Vec::new(),
        };

        // Get all active platform accounts for the user
        let accounts = git_platform_account::Entity::find()
            .filter(git_platform_account::Column::UserId.eq(user_id))
            .filter(git_platform_account::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        if accounts.is_empty() {
            log::info!("No active platform accounts found for user {}", user_id);
            return Ok(result);
        }

        // Calculate current year date range
        let now = Utc::now();
        let current_year = now.year();
        let start_date = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1)
            .context("Invalid start date")?;
        let end_date = now.date_naive();

        // Sync each platform account
        for account in &accounts {
            // First, sync profile data if enabled
            if account.sync_profile {
                if let Err(e) = self.sync_profile_data(account).await {
                    let error_msg = format!(
                        "Failed to sync profile for {}: {}",
                        account.platform_username, e
                    );
                    log::error!("{}", error_msg);
                    result.errors.push(error_msg);
                }
            } else {
                log::debug!("Profile sync disabled for {}", account.platform_username);
            }

            // Then sync contribution data if enabled
            if account.sync_contributions {
                match self.sync_platform_account(account, start_date, end_date).await {
                Ok(stats) => {
                    result.platforms_synced += 1;
                    result.contributions_added += stats.added;
                    result.contributions_updated += stats.updated;
                    log::info!(
                        "Synced platform {} for user {}: {} added, {} updated",
                        account.platform_username,
                        user_id,
                        stats.added,
                        stats.updated
                    );
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to sync platform {}: {}",
                        account.platform_username, e
                    );
                    log::error!("{}", error_msg);
                    result.errors.push(error_msg);
                }
            }
        } else {
            log::debug!("Contribution sync disabled for {}", account.platform_username);
        }
        }

        // Update last_synced_at for all accounts
        if result.platforms_synced > 0 {
            for account in accounts {
                let mut active_account: git_platform_account::ActiveModel = account.into();
                active_account.last_synced_at = Set(Some(Utc::now()));
                let _ = active_account.update(&self.db).await;
            }
        }

        // If data was updated, regenerate heatmaps
        if result.contributions_added > 0 || result.contributions_updated > 0 {
            log::info!("Data updated, regenerating heatmaps for user {}", user_id);
            if let Err(e) = self.regenerate_all_heatmaps(user_id).await {
                let error_msg = format!("Failed to regenerate heatmaps: {}", e);
                log::error!("{}", error_msg);
                result.errors.push(error_msg);
            }
        }

        log::info!(
            "Sync completed for user {}: {} platforms synced, {} contributions added, {} updated",
            user_id,
            result.platforms_synced,
            result.contributions_added,
            result.contributions_updated
        );

        Ok(result)
    }

    /// Sync a single platform account (unified: contributions + activities)
    async fn sync_platform_account(
        &self,
        account: &git_platform_account::Model,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<SyncStats> {
        let mut stats = SyncStats {
            added: 0,
            updated: 0,
        };

        // ========================================
        // STEP 1: Delete existing contributions in the date range
        // ========================================
        log::info!("🗑️  Deleting existing contributions from {} to {}", start_date, end_date);

        let deleted = contribution::Entity::delete_many()
            .filter(contribution::Column::GitPlatformAccountId.eq(account.id))
            .filter(contribution::Column::ContributionDate.gte(start_date))
            .filter(contribution::Column::ContributionDate.lte(end_date))
            .exec(&self.db)
            .await?;

        log::info!("🗑️  Deleted {} existing contributions in date range", deleted.rows_affected);

        // ========================================
        // STEP 2: Fetch fresh contributions from platform
        // ========================================
        let contributions = self.fetch_contributions_from_platform(account, start_date, end_date).await?;

        log::info!("📊 Fetched {} fresh contribution records", contributions.len());

        // ========================================
        // STEP 3: Insert all fresh contributions
        // ========================================
        for contrib in &contributions {
            let new_contrib = contribution::ActiveModel {
                id: Set(Uuid::new_v4()),
                git_platform_account_id: Set(account.id),
                contribution_date: Set(contrib.date),
                count: Set(contrib.count),
                repository_name: Set(contrib.repository_name.clone()),
                is_private_repo: Set(contrib.is_private),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            contribution::Entity::insert(new_contrib).exec(&self.db).await?;
            stats.added += 1;
        }

        log::info!("✅ Inserted {} fresh contributions", stats.added);

        // ========================================
        // PART 2: Sync activities using the same contribution data
        // ========================================
        if account.sync_activities {
            log::info!("📅 Syncing activities for timeline...");

            let from = chrono::NaiveDateTime::new(start_date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_utc();
            let to = chrono::NaiveDateTime::new(end_date, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap()).and_utc();

            if let Err(e) = self.sync_activities_from_contributions(account, &contributions, from, to).await {
                log::error!("Failed to sync activities: {}", e);
            }
        } else {
            log::debug!("Activity sync disabled for {}", account.platform_username);
        }

        Ok(stats)
    }

    /// Sync profile data from git platform
    async fn sync_profile_data(&self, account: &git_platform_account::Model) -> Result<()> {
        // Fetch profile data from platform
        let profile = self.fetch_profile_from_platform(account).await?;

        // Update account with profile data
        let mut active_account: git_platform_account::ActiveModel = account.clone().into();

        if let Some(avatar) = profile.avatar_url {
            active_account.avatar_url = Set(Some(avatar));
        }
        if let Some(name) = profile.display_name {
            active_account.display_name = Set(Some(name));
        }
        if let Some(bio) = profile.bio {
            active_account.bio = Set(Some(bio));
        }
        if let Some(url) = profile.profile_url {
            active_account.profile_url = Set(Some(url));
        }
        if let Some(location) = profile.location {
            active_account.location = Set(Some(location));
        }
        if let Some(company) = profile.company {
            active_account.company = Set(Some(company));
        }
        if let Some(followers) = profile.followers_count {
            active_account.followers_count = Set(Some(followers));
        }
        if let Some(following) = profile.following_count {
            active_account.following_count = Set(Some(following));
        }

        active_account.updated_at = Set(Utc::now());
        active_account.update(&self.db).await?;

        log::info!("Updated profile for {}", account.platform_username);
        Ok(())
    }

    /// Fetch contributions from git platform
    async fn fetch_contributions_from_platform(
        &self,
        account: &git_platform_account::Model,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<Contribution>> {
        // Decrypt access token
        let access_token = account.access_token.as_ref()
            .context("No access token found")?;
        let decrypted_token = encryption::decrypt(access_token, &self.config.encryption_key)
            .context("Failed to decrypt access token")?;

        // Convert dates to DateTime<Utc>
        let from = chrono::NaiveDateTime::new(
            start_date,
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        ).and_utc();
        let to = chrono::NaiveDateTime::new(
            end_date,
            chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap()
        ).and_utc();

        // Fetch contributions based on platform type
        let contributions = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                let client = GitHubClient::new();
                let config = PlatformConfig::github();

                log::info!("Fetching GitHub contributions for {}", account.platform_username);

                client.fetch_contributions(&config, &account.platform_username, &decrypted_token, from, to)
                    .await
                    .context("Failed to fetch GitHub contributions")?
            }
            git_platform_account::GitPlatform::Gitea => {
                let client = GiteaClient::new();
                let instance_url = account.platform_url.as_ref()
                    .context("Gitea instance URL not found")?;
                let config = PlatformConfig::gitea_custom(instance_url);

                log::info!("Fetching Gitea contributions for {} from {}", account.platform_username, instance_url);

                client.fetch_contributions(&config, &account.platform_username, &decrypted_token, from, to)
                    .await
                    .context("Failed to fetch Gitea contributions")?
            }
            git_platform_account::GitPlatform::GitLab => {
                log::warn!("GitLab sync not yet implemented");
                return Ok(Vec::new());
            }
        };

        log::info!("Fetched {} contribution days for {}", contributions.len(), account.platform_username);

        Ok(contributions)
    }

    /// Fetch profile data from git platform
    async fn fetch_profile_from_platform(
        &self,
        account: &git_platform_account::Model,
    ) -> Result<ProfileData> {
        // Decrypt access token
        let access_token = account.access_token.as_ref()
            .context("No access token found")?;
        let decrypted_token = encryption::decrypt(access_token, &self.config.encryption_key)
            .context("Failed to decrypt access token")?;

        // Fetch profile based on platform type
        let profile_data = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                let client = GitHubClient::new();
                let config = PlatformConfig::github();

                log::info!("Fetching GitHub profile for {}", account.platform_username);

                let profile = client.fetch_user_profile(&config, &account.platform_username, &decrypted_token)
                    .await
                    .context("Failed to fetch GitHub profile")?;

                ProfileData {
                    avatar_url: profile.get("avatar_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    display_name: profile.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    bio: profile.get("bio").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    profile_url: profile.get("html_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    location: profile.get("location").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    company: profile.get("company").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    followers_count: profile.get("followers").and_then(|v| v.as_i64()).map(|n| n as i32),
                    following_count: profile.get("following").and_then(|v| v.as_i64()).map(|n| n as i32),
                }
            }
            git_platform_account::GitPlatform::Gitea => {
                let client = GiteaClient::new();
                let instance_url = account.platform_url.as_ref()
                    .context("Gitea instance URL not found")?;
                let config = PlatformConfig::gitea_custom(instance_url);

                log::info!("Fetching Gitea profile for {} from {}", account.platform_username, instance_url);

                let profile = client.fetch_user_profile(&config, &decrypted_token)
                    .await
                    .context("Failed to fetch Gitea profile")?;

                ProfileData {
                    avatar_url: profile.get("avatar_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    display_name: profile.get("full_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    bio: profile.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    profile_url: Some(format!("{}/{}", instance_url, account.platform_username)),
                    location: profile.get("location").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    company: None, // Gitea doesn't have company field
                    followers_count: profile.get("followers_count").and_then(|v| v.as_i64()).map(|n| n as i32),
                    following_count: profile.get("following_count").and_then(|v| v.as_i64()).map(|n| n as i32),
                }
            }
            git_platform_account::GitPlatform::GitLab => {
                log::warn!("GitLab profile sync not yet implemented");
                return Ok(ProfileData::default());
            }
        };

        Ok(profile_data)
    }

    /// Sync activities from contributions (unified approach - no duplicate API calls)
    async fn sync_activities_from_contributions(
        &self,
        account: &git_platform_account::Model,
        contributions: &[Contribution],
        from: chrono::DateTime<Utc>,
        to: chrono::DateTime<Utc>,
    ) -> Result<()> {
        let from_date = from.naive_utc().date();
        let to_date = to.naive_utc().date();

        // Delete existing activities in the date range
        let deleted = activity::Entity::delete_many()
            .filter(activity::Column::GitPlatformAccountId.eq(account.id))
            .filter(activity::Column::ActivityDate.gte(from_date))
            .filter(activity::Column::ActivityDate.lte(to_date))
            .exec(&self.db)
            .await?;

        log::info!("🗑️  Deleted {} existing activities in date range", deleted.rows_affected);

        // Aggregate commits by month from contribution data (no additional API calls!)
        let mut commits_by_month: HashMap<(i32, u32), (Vec<serde_json::Value>, i32, bool, chrono::NaiveDate)> = HashMap::new();

        for contribution in contributions {
            if contribution.count > 0 {
                // Skip contributions without repository names
                let Some(ref repo_name) = contribution.repository_name else {
                    log::debug!("Skipping contribution on {} - no repository name available", contribution.date);
                    continue;
                };

                let year = contribution.date.year();
                let month = contribution.date.month();
                let month_key = (year, month);

                let entry = commits_by_month.entry(month_key).or_insert((Vec::new(), 0, false, contribution.date));

                // Use the latest date in the month for sorting
                if contribution.date > entry.3 {
                    entry.3 = contribution.date;
                }

                // Check if this repo already exists in this month's aggregation
                let repo_exists = entry.0.iter_mut().find(|r| {
                    r.get("name").and_then(|v| v.as_str()) == Some(repo_name)
                });

                if let Some(existing_repo) = repo_exists {
                    // Add to existing repository's commit count
                    if let Some(count) = existing_repo.get_mut("commit_count") {
                        if let Some(current_count) = count.as_i64() {
                            *count = json!(current_count + contribution.count as i64);
                        }
                    }
                } else {
                    // Add new repository to the list
                    entry.0.push(json!({
                        "name": repo_name,
                        "commit_count": contribution.count,
                    }));
                }

                entry.1 += contribution.count;

                if contribution.is_private {
                    entry.2 = true;
                }
            }
        }

        log::info!("📊 Aggregated contributions into {} month activities", commits_by_month.len());

        // Store commit activities
        for ((year, month), (repos, total_count, has_private, latest_date)) in commits_by_month {
            let activity_model = activity::ActiveModel {
                id: Set(Uuid::new_v4()),
                git_platform_account_id: Set(account.id),
                activity_type: Set(activity::ActivityType::Commit),
                activity_date: Set(latest_date),
                metadata: Set(json!({
                    "repositories": repos,
                    "total_count": total_count,
                    "year": year,
                    "month": month,
                })),
                repository_name: Set(None),
                repository_url: Set(None),
                is_private_repo: Set(has_private),
                count: Set(total_count),
                primary_language: Set(None),
                organization_name: Set(None),
                organization_avatar_url: Set(None),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            activity::Entity::insert(activity_model).exec(&self.db).await?;
        }

        // Fetch additional activity types (repos created, PRs, issues, orgs) - only for GitHub for now
        if matches!(account.platform_type, git_platform_account::GitPlatform::GitHub) {
            let client = GitHubClient::new();
            let config = PlatformConfig::github();

            let access_token = account.access_token.as_ref()
                .context("No access token found")?;
            let token = encryption::decrypt(access_token, &self.config.encryption_key)
                .context("Failed to decrypt access token")?;

            // Fetch repository creation activities
            log::info!("📦 Fetching repository creation activities...");
            if let Ok(repo_activities) = client.fetch_repository_creation_activities(
                &config, &account.platform_username, &token, from, to
            ).await {
                log::info!("Found {} repository creation activities", repo_activities.len());
                for activity in repo_activities {
                    self.store_activity(account.id, activity).await?;
                }
            }

            // Fetch PR and issue activities
            log::info!("🔀 Fetching PR and issue activities...");
            if let Ok(pr_issue_activities) = client.fetch_pr_and_issue_activities(
                &config, &account.platform_username, &token, from, to
            ).await {
                log::info!("Found {} PR/issue activities", pr_issue_activities.len());
                for activity in pr_issue_activities {
                    self.store_activity(account.id, activity).await?;
                }
            }
        }

        log::info!("✅ Activities sync completed");

        Ok(())
    }

    /// Store a single activity
    async fn store_activity(&self, account_id: Uuid, activity: Activity) -> Result<()> {
        let db_activity_type = match activity.activity_type {
            ActivityType::Commit => activity::ActivityType::Commit,
            ActivityType::RepositoryCreated => activity::ActivityType::RepositoryCreated,
            ActivityType::PullRequest => activity::ActivityType::PullRequest,
            ActivityType::Issue => activity::ActivityType::Issue,
            ActivityType::Review => activity::ActivityType::Review,
            ActivityType::OrganizationJoined => activity::ActivityType::OrganizationJoined,
            ActivityType::Fork => activity::ActivityType::Fork,
            ActivityType::Release => activity::ActivityType::Release,
            ActivityType::Star => activity::ActivityType::Star,
        };

        let activity_model = activity::ActiveModel {
            id: Set(Uuid::new_v4()),
            git_platform_account_id: Set(account_id),
            activity_type: Set(db_activity_type),
            activity_date: Set(activity.date),
            metadata: Set(activity.metadata),
            repository_name: Set(activity.repository_name),
            repository_url: Set(activity.repository_url),
            is_private_repo: Set(activity.is_private),
            count: Set(activity.count),
            primary_language: Set(activity.primary_language),
            organization_name: Set(activity.organization_name),
            organization_avatar_url: Set(activity.organization_avatar_url),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        activity::Entity::insert(activity_model).exec(&self.db).await?;

        Ok(())
    }

    /// Regenerate all heatmaps for a user's themes
    async fn regenerate_all_heatmaps(&self, user_id: Uuid) -> Result<()> {
        let generator = HeatmapGenerator::new(self.db.clone());

        // Get all themes for the user
        let themes = heatmap_theme::Entity::find()
            .filter(heatmap_theme::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;

        log::info!("Regenerating {} themes for user {}", themes.len(), user_id);

        for theme in themes {
            match generator.generate_for_theme(user_id, &theme).await {
                Ok(_) => {
                    log::info!("Regenerated theme: {}", theme.slug);
                }
                Err(e) => {
                    log::error!("Failed to regenerate theme {}: {}", theme.slug, e);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
struct ProfileData {
    avatar_url: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    profile_url: Option<String>,
    location: Option<String>,
    company: Option<String>,
    followers_count: Option<i32>,
    following_count: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub platforms_synced: i32,
    pub contributions_added: i32,
    pub contributions_updated: i32,
    pub errors: Vec<String>,
}

#[derive(Debug)]
struct SyncStats {
    added: i32,
    updated: i32,
}
