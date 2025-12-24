use crate::models::activity::{ActiveModel as ActivityActiveModel, ActivityType as DbActivityType};
use crate::models::git_platform_account;
use crate::services::git_platforms::{Activity, ActivityType, GitHubClient, GiteaClient, GitPlatform, PlatformConfig};
use crate::utils::encryption;
use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;

pub struct ActivityAggregationService {
    db: DatabaseConnection,
    encryption_key: String,
}

impl ActivityAggregationService {
    pub fn new(db: DatabaseConnection, encryption_key: String) -> Self {
        Self { db, encryption_key }
    }

    /// Fetch and store activities for all platform accounts of a user
    pub async fn sync_user_activities(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<()> {
        // Fetch all active platform accounts for the user
        let accounts = git_platform_account::Entity::find()
            .filter(git_platform_account::Column::UserId.eq(user_id))
            .filter(git_platform_account::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        log::info!("Syncing activities for {} platform accounts", accounts.len());

        for account in accounts {
            if let Err(e) = self.sync_platform_activities(&account, from, to).await {
                log::error!(
                    "Failed to sync activities for platform account {}: {}",
                    account.id,
                    e
                );
            }
        }

        Ok(())
    }

    /// Fetch and store activities for a single specific platform account
    pub async fn sync_single_platform_activity(
        &self,
        platform_account_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<()> {
        // Fetch the specific platform account
        let account = git_platform_account::Entity::find_by_id(platform_account_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Platform account not found"))?;

        if !account.is_active {
            return Err(anyhow::anyhow!("Platform account is not active"));
        }

        log::info!("Syncing activities for single platform account: {} ({})",
            account.platform_username, account.id);

        self.sync_platform_activities(&account, from, to).await
    }

    /// Fetch and store activities for a specific platform account
    async fn sync_platform_activities(
        &self,
        account: &git_platform_account::Model,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<()> {
        let platform_client = self.get_platform_client(&account.platform_type);
        let config = self.get_platform_config(&account.platform_type, account.platform_url.as_deref());

        let encrypted_token = account
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No access token for account"))?;

        // Decrypt the token before using it
        let token = encryption::decrypt(encrypted_token, &self.encryption_key)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt access token: {}", e))?;

        log::info!(
            "Fetching activities for {:?} account: {}",
            account.platform_type,
            account.platform_username
        );

        // Delete existing activities in the date range to avoid duplicates
        let from_date = from.naive_utc().date();
        let to_date = to.naive_utc().date();

        let deleted = crate::models::activity::Entity::delete_many()
            .filter(crate::models::activity::Column::GitPlatformAccountId.eq(account.id))
            .filter(crate::models::activity::Column::ActivityDate.gte(from_date))
            .filter(crate::models::activity::Column::ActivityDate.lte(to_date))
            .exec(&self.db)
            .await?;

        log::info!("🗑️  Deleted {} existing activities in date range", deleted.rows_affected);

        // Fetch contributions for accurate commit counts
        log::info!("Fetching contributions for commit counts...");

        // For GitHub, we need to fetch year by year due to 1-year API limit
        let contributions = if matches!(account.platform_type, git_platform_account::GitPlatform::GitHub) {
            let start_year = from.year();
            let end_year = to.year();

            let mut all_contributions = Vec::new();

            for year in start_year..=end_year {
                let year_from = if year == start_year {
                    from // Use the actual start date for the first year
                } else {
                    chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc()
                };

                let year_to = if year == end_year {
                    to // Use the actual end date for the last year
                } else {
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31)
                        .unwrap()
                        .and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_utc()
                };

                log::info!("Fetching contributions for year {}: {} to {}",
                    year, year_from.format("%Y-%m-%d"), year_to.format("%Y-%m-%d"));

                let year_contributions = platform_client
                    .fetch_contributions(&config, &account.platform_username, &token, year_from, year_to)
                    .await?;

                log::info!("✅ Fetched {} contribution days for year {}", year_contributions.len(), year);
                all_contributions.extend(year_contributions);
            }

            all_contributions
        } else {
            // For other platforms (Gitea, GitLab), fetch all at once
            platform_client
                .fetch_contributions(&config, &account.platform_username, &token, from, to)
                .await?
        };

        let contribution_count = contributions.len();
        log::info!("Fetched {} contribution days (total across all years)", contribution_count);

        // Aggregate contributions by MONTH (not just date) to avoid pagination issues
        use std::collections::HashMap;
        let mut commits_by_month: HashMap<(i32, u32), (Vec<serde_json::Value>, i32, bool, chrono::NaiveDate)> = HashMap::new();

        for contribution in contributions {
            if contribution.count > 0 {
                let repo_name = contribution.repository_name.clone().unwrap_or_else(|| "Unknown".to_string());

                // Group by year and month
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
                    r.get("name").and_then(|v| v.as_str()) == Some(&repo_name)
                });

                if let Some(existing_repo) = repo_exists {
                    // Add to existing repository's commit count
                    if let Some(count) = existing_repo.get_mut("commit_count") {
                        if let Some(current_count) = count.as_i64() {
                            *count = serde_json::json!(current_count + contribution.count as i64);
                        }
                    }
                } else {
                    // Add new repository to the list
                    entry.0.push(serde_json::json!({
                        "name": repo_name,
                        "commit_count": contribution.count,
                    }));
                }

                // Add to total count
                entry.1 += contribution.count;

                // Track if any repo is private
                if contribution.is_private {
                    entry.2 = true;
                }
            }
        }

        log::info!("Aggregated {} contribution days into {} months", contribution_count, commits_by_month.len());

        // Convert aggregated data to commit activities (one activity per MONTH)
        let mut commit_activities = Vec::new();
        for ((year, month), (repos, total_count, has_private, latest_date)) in commits_by_month {
            commit_activities.push(Activity {
                activity_type: ActivityType::Commit,
                date: latest_date, // Use latest date in month for sorting
                metadata: serde_json::json!({
                    "repositories": repos,
                    "total_count": total_count,
                    "year": year,
                    "month": month,
                }),
                repository_name: None, // No single repo name since this is aggregated
                repository_url: None,
                is_private: has_private,
                count: total_count,
                primary_language: None,
                organization_name: None,
                organization_avatar_url: None,
            });
        }

        log::info!("Created {} month-aggregated commit activities", commit_activities.len());

        // Fetch repository creation activities from GraphQL (works for all history)
        log::info!("Fetching repository creation activities from GraphQL...");
        let repo_creation_activities = platform_client
            .fetch_repository_creation_activities(&config, &account.platform_username, &token, from, to)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to fetch repository creation activities: {}", e);
                Vec::new()
            });

        log::info!("Fetched {} repository creation activities from GraphQL", repo_creation_activities.len());

        // Fetch PR and issue activities from GraphQL search (works for all history)
        // Note: Currently only implemented for GitHub
        log::info!("Fetching PR and issue activities from GraphQL search...");
        let pr_issue_activities = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                let github_client = GitHubClient::new();
                github_client
                    .fetch_pr_and_issue_activities(&config, &account.platform_username, &token, from, to)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to fetch PR/issue activities: {}", e);
                        Vec::new()
                    })
            }
            _ => Vec::new(), // Other platforms not yet supported
        };

        log::info!("Fetched {} PR/issue activities from GraphQL", pr_issue_activities.len());

        // Fetch organization memberships and detect new joins (GitHub only)
        log::info!("Fetching organization memberships...");
        let organization_activities = match account.platform_type {
            git_platform_account::GitPlatform::GitHub => {
                let github_client = GitHubClient::new();

                // Fetch current public organizations
                let current_orgs = github_client
                    .fetch_user_organizations(&config, &account.platform_username, &token)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to fetch organizations: {}", e);
                        Vec::new()
                    });

                // Delete ALL existing organization join activities for this account
                // We'll recreate them with correct dates from scraping/events
                let deleted = crate::models::activity::Entity::delete_many()
                    .filter(crate::models::activity::Column::GitPlatformAccountId.eq(account.id))
                    .filter(crate::models::activity::Column::ActivityType.eq(DbActivityType::OrganizationJoined))
                    .exec(&self.db)
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to delete existing org activities: {}", e);
                        sea_orm::DeleteResult { rows_affected: 0 }
                    });

                log::info!("🗑️  Deleted {} existing organization join activities (will recreate with correct dates)",
                    deleted.rows_affected);

                log::info!("Found {} current organizations", current_orgs.len());

                // Create activities for ALL current organizations with correct dates
                let mut new_org_activities = Vec::new();
                let today = chrono::Utc::now().naive_utc().date();

                for (org_name, avatar_url) in current_orgs {
                    // Try to find the join date using web scraping first (most accurate)
                    let mut join_date_opt = github_client
                        .scrape_org_join_date(&account.platform_username, &org_name)
                        .await
                        .ok()
                        .flatten();

                    // If scraping failed, try finding earliest activity as fallback
                    if join_date_opt.is_none() {
                        log::info!("Scraping failed, trying event API for org {}", org_name);
                        join_date_opt = github_client
                            .find_earliest_org_activity_date(&config, &account.platform_username, &org_name)
                            .await
                            .ok()
                            .flatten();
                    }

                    let join_date = join_date_opt.unwrap_or_else(|| {
                        log::warn!("⚠️  No join date found for org {}, using current date", org_name);
                        today
                    });

                    log::info!("📅 Creating org join activity: {} on {}", org_name, join_date);

                    new_org_activities.push(Activity {
                        activity_type: ActivityType::OrganizationJoined,
                        date: join_date,
                        metadata: serde_json::json!({
                            "organization": org_name,
                            "avatar_url": avatar_url,
                            "joined_at": join_date.format("%Y-%m-%d").to_string(),
                        }),
                        repository_name: None,
                        repository_url: None,
                        is_private: false,
                        count: 1,
                        primary_language: None,
                        organization_name: Some(org_name.clone()),
                        organization_avatar_url: Some(avatar_url),
                    });
                }

                log::info!("Created {} organization join activities with scraped dates", new_org_activities.len());
                new_org_activities
            }
            _ => Vec::new(), // Other platforms not yet supported
        };

        // Fetch remaining activity types from Events API (forks, stars, etc.)
        let events_activities = platform_client
            .fetch_activities(&config, &account.platform_username, &token, from, to)
            .await?;

        // Filter out commits, repos, PRs, issues, and org joins from Events API (we have GraphQL data for these)
        let mut activities: Vec<_> = events_activities.into_iter()
            .filter(|a| {
                a.activity_type != ActivityType::Commit
                && a.activity_type != ActivityType::RepositoryCreated
                && a.activity_type != ActivityType::PullRequest
                && a.activity_type != ActivityType::Issue
                && a.activity_type != ActivityType::OrganizationJoined
            })
            .collect();

        log::info!("Fetched {} other activities from Events API", activities.len());

        // Combine all activities
        activities.extend(commit_activities);
        activities.extend(repo_creation_activities);
        activities.extend(pr_issue_activities);
        activities.extend(organization_activities);

        log::info!("Total activities to store: {}", activities.len());

        // Store activities in database
        let mut stored_count = 0;
        for activity in activities {
            match self.store_activity(&account.id, activity).await {
                Ok(_) => stored_count += 1,
                Err(e) => log::error!("Failed to store activity: {}", e),
            }
        }

        log::info!("✅ Stored {} activities in database", stored_count);

        Ok(())
    }

    /// Store a single activity in the database
    async fn store_activity(&self, account_id: &Uuid, activity: Activity) -> Result<()> {
        // Convert ActivityType to DbActivityType
        let db_activity_type = match activity.activity_type {
            ActivityType::Commit => DbActivityType::Commit,
            ActivityType::RepositoryCreated => DbActivityType::RepositoryCreated,
            ActivityType::PullRequest => DbActivityType::PullRequest,
            ActivityType::Issue => DbActivityType::Issue,
            ActivityType::Review => DbActivityType::Review,
            ActivityType::OrganizationJoined => DbActivityType::OrganizationJoined,
            ActivityType::Fork => DbActivityType::Fork,
            ActivityType::Release => DbActivityType::Release,
            ActivityType::Star => DbActivityType::Star,
        };

        let activity_model = ActivityActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            git_platform_account_id: ActiveValue::Set(*account_id),
            activity_type: ActiveValue::Set(db_activity_type),
            activity_date: ActiveValue::Set(activity.date),
            metadata: ActiveValue::Set(activity.metadata),
            repository_name: ActiveValue::Set(activity.repository_name),
            repository_url: ActiveValue::Set(activity.repository_url),
            is_private_repo: ActiveValue::Set(activity.is_private),
            count: ActiveValue::Set(activity.count),
            primary_language: ActiveValue::Set(activity.primary_language),
            organization_name: ActiveValue::Set(activity.organization_name),
            organization_avatar_url: ActiveValue::Set(activity.organization_avatar_url),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            updated_at: ActiveValue::Set(chrono::Utc::now()),
        };

        activity_model.insert(&self.db).await?;
        Ok(())
    }

    /// Get the appropriate platform client based on platform type
    fn get_platform_client(
        &self,
        platform_type: &git_platform_account::GitPlatform,
    ) -> Box<dyn GitPlatform> {
        match platform_type {
            git_platform_account::GitPlatform::GitHub => Box::new(GitHubClient::new()),
            git_platform_account::GitPlatform::GitLab => {
                // TODO: Implement GitLab client
                unimplemented!("GitLab not yet implemented")
            }
            git_platform_account::GitPlatform::Gitea => Box::new(GiteaClient::new()),
        }
    }

    /// Get platform configuration based on platform type and custom URL
    fn get_platform_config(
        &self,
        platform_type: &git_platform_account::GitPlatform,
        platform_url: Option<&str>,
    ) -> PlatformConfig {
        match platform_type {
            git_platform_account::GitPlatform::GitHub => PlatformConfig::github(),
            git_platform_account::GitPlatform::GitLab => {
                if let Some(url) = platform_url {
                    PlatformConfig::gitlab_custom(url)
                } else {
                    PlatformConfig::gitlab()
                }
            }
            git_platform_account::GitPlatform::Gitea => {
                PlatformConfig::gitea_custom(platform_url.unwrap_or(""))
            }
        }
    }
}
