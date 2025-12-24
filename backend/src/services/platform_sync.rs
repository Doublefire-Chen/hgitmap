use anyhow::{Context, Result};
use chrono::{Datelike, Utc};
use sea_orm::*;
use uuid::Uuid;

use crate::models::{contribution, git_platform_account, heatmap_theme};
use crate::services::heatmap_generator::HeatmapGenerator;

pub struct PlatformSyncService {
    db: DatabaseConnection,
}

impl PlatformSyncService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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

    /// Sync a single platform account
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

        // Generate mock data for now (replace with actual API calls)
        let contributions = self.fetch_contributions_from_platform(account, start_date, end_date).await?;

        // Update contributions in database
        for (date, count) in contributions {
            // Check if contribution already exists
            let existing = contribution::Entity::find()
                .filter(contribution::Column::GitPlatformAccountId.eq(account.id))
                .filter(contribution::Column::ContributionDate.eq(date))
                .one(&self.db)
                .await?;

            match existing {
                Some(existing_contrib) => {
                    // Update if count changed
                    if existing_contrib.count != count {
                        let mut active_contrib: contribution::ActiveModel = existing_contrib.into();
                        active_contrib.count = Set(count);
                        active_contrib.updated_at = Set(Utc::now());
                        active_contrib.update(&self.db).await?;
                        stats.updated += 1;
                    }
                }
                None => {
                    // Insert new contribution
                    let new_contrib = contribution::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        git_platform_account_id: Set(account.id),
                        contribution_date: Set(date),
                        count: Set(count),
                        repository_name: Set(None),
                        is_private_repo: Set(false),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                    };
                    contribution::Entity::insert(new_contrib).exec(&self.db).await?;
                    stats.added += 1;
                }
            }
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

    /// Fetch contributions from git platform (placeholder - needs real implementation)
    async fn fetch_contributions_from_platform(
        &self,
        account: &git_platform_account::Model,
        _start_date: chrono::NaiveDate,
        _end_date: chrono::NaiveDate,
    ) -> Result<Vec<(chrono::NaiveDate, i32)>> {
        // TODO: Implement actual API calls to GitHub, GitLab, Gitea
        // For now, return empty list to avoid errors
        // This is where you would:
        // 1. Use the platform's API client (GitHub GraphQL, GitLab REST, etc.)
        // 2. Fetch contribution data for the date range
        // 3. Parse and return the data

        log::warn!(
            "Platform sync not implemented for {:?}: {}",
            account.platform_type,
            account.platform_username
        );

        Ok(Vec::new())
    }

    /// Fetch profile data from git platform (placeholder - needs real implementation)
    async fn fetch_profile_from_platform(
        &self,
        account: &git_platform_account::Model,
    ) -> Result<ProfileData> {
        // TODO: Implement actual API calls to GitHub, GitLab, Gitea
        // For now, return empty profile data
        // This is where you would:
        // 1. Use the platform's API client
        // 2. Fetch user profile information
        // 3. Parse and return the data

        log::warn!(
            "Profile sync not implemented for {:?}: {}",
            account.platform_type,
            account.platform_username
        );

        Ok(ProfileData::default())
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
