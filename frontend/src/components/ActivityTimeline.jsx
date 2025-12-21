import { useState, useEffect, useCallback } from 'react';
import apiClient from '../api/client';
import ActivityIcons from './ActivityIcons';
import './ActivityTimeline.css';

function ActivityTimeline() {
  const [activities, setActivities] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(0);
  const limit = 20;

  const loadActivities = useCallback(async (newOffset = 0) => {
    try {
      setLoading(true);
      setError(null);

      console.log('🔍 Loading activities with offset:', newOffset);
      const data = await apiClient.getActivities(null, null, limit, newOffset);
      console.log('📦 Received activity data:', data);

      if (newOffset === 0) {
        // Initial load
        setActivities(data.activities || []);
      } else {
        // Load more
        setActivities(prev => [...prev, ...(data.activities || [])]);
      }

      setHasMore(data.has_more);
      setOffset(newOffset);

      console.log(`✅ Loaded ${data.activities?.length || 0} activities (total: ${data.total})`);
    } catch (err) {
      console.error('❌ Failed to load activities:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [limit]);

  useEffect(() => {
    loadActivities(0);
  }, [loadActivities]);

  const loadMore = () => {
    if (hasMore && !loading) {
      loadActivities(offset + limit);
    }
  };

  const formatDate = (dateStr) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffTime = Math.abs(now - date);
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;

    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    return `${months[date.getMonth()]} ${date.getDate()}`;
  };

  const groupActivitiesByMonth = () => {
    const groups = {};

    // Track processed activity IDs to avoid double-counting
    const processedActivityIds = new Set();

    console.log('📊 Total activities to process:', activities.length);

    activities.forEach(activity => {
      // Skip if we've already processed this activity
      if (processedActivityIds.has(activity.id)) {
        console.log('⚠️  Skipping duplicate activity ID:', activity.id, activity.activity_type);
        return;
      }
      processedActivityIds.add(activity.id);

      const date = new Date(activity.date);
      const monthYear = `${date.toLocaleString('default', { month: 'long' })} ${date.getFullYear()}`;

      if (!groups[monthYear]) {
        groups[monthYear] = [];
      }

      groups[monthYear].push(activity);
    });

    return groups;
  };

  const getActivityIcon = (type) => {
    const IconComponent = ActivityIcons[type] || ActivityIcons.Default;
    return <IconComponent />;
  };

  const getCommitBarColor = (commitCount, maxCommits) => {
    // Calculate intensity similar to heatmap (0-4 levels)
    const percentage = commitCount / maxCommits;

    // Check if dark theme is active
    const isDark = document.documentElement.getAttribute('data-theme') === 'dark';

    // Use GitHub's contribution colors (light theme)
    if (!isDark) {
      if (percentage >= 0.75) return '#2da44e'; // level-3
      if (percentage >= 0.50) return '#4ac26b'; // level-2
      if (percentage >= 0.25) return '#aceebb'; // level-1
      return '#9be9a8'; // light green for smallest
    }

    // Dark theme colors
    if (percentage >= 0.75) return '#2ea043'; // level-3
    if (percentage >= 0.50) return '#196c2e'; // level-2
    if (percentage >= 0.25) return '#033a16'; // level-1
    return '#0d4429'; // darker green for smallest
  };

  const formatActivityType = (type) => {
    // Convert from PascalCase to space-separated
    return type.replace(/([A-Z])/g, ' $1').trim();
  };

  const renderActivity = (activity) => {
    const metadata = activity.metadata;

    switch (activity.activity_type) {
      case 'Commit': {
        const repos = metadata.repositories || [];
        const totalCommits = activity.count;
        const maxCommits = Math.max(...repos.map(r => r.commit_count));

        return (
          <div className="activity-item" key={activity.id}>
            <div className="activity-icon">{getActivityIcon(activity.activity_type)}</div>
            <div className="activity-content">
              <div className="activity-header">
                Created {totalCommits} commit{totalCommits !== 1 ? 's' : ''} in {repos.length} {repos.length === 1 ? 'repository' : 'repositories'}
              </div>
              <div className="activity-details">
                {repos.map((repo, idx) => (
                  <div key={idx} className="repository-item">
                    <a href={`https://github.com/${repo.name}`} target="_blank" rel="noopener noreferrer">
                      {repo.name}
                    </a>
                    <span className="commit-count">{repo.commit_count} commits</span>
                    <div className="commit-bar">
                      <div
                        className="commit-fill"
                        style={{
                          width: `${(repo.commit_count / totalCommits) * 100}%`,
                          backgroundColor: getCommitBarColor(repo.commit_count, maxCommits)
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
              <div className="activity-date">{formatDate(activity.date)}</div>
            </div>
          </div>
        );
      }

      case 'RepositoryCreated':
        return (
          <div className="activity-item" key={activity.id}>
            <div className="activity-icon">{getActivityIcon(activity.activity_type)}</div>
            <div className="activity-content">
              <div className="activity-header">Created 1 repository</div>
              <div className="activity-details">
                <div className="repository-item">
                  <a href={activity.repository_url} target="_blank" rel="noopener noreferrer">
                    {activity.repository_name}
                  </a>
                  {activity.primary_language && (
                    <span className="language-tag" data-language={activity.primary_language}>
                      {activity.primary_language}
                    </span>
                  )}
                </div>
              </div>
              <div className="activity-date">on {formatDate(activity.date)}</div>
            </div>
          </div>
        );

      case 'OrganizationJoined':
        return (
          <div className="activity-item" key={activity.id}>
            <div className="activity-icon">{getActivityIcon(activity.activity_type)}</div>
            <div className="activity-content">
              <div className="activity-header">Joined the {activity.organization_name} organization</div>
              <div className="activity-date">on {formatDate(activity.date)}</div>
            </div>
          </div>
        );

      case 'PullRequest':
      case 'Issue':
        return (
          <div className="activity-item" key={activity.id}>
            <div className="activity-icon">{getActivityIcon(activity.activity_type)}</div>
            <div className="activity-content">
              <div className="activity-header">
                {activity.activity_type === 'PullRequest' ? 'Opened pull request' : 'Opened issue'} in{' '}
                <a href={activity.repository_url} target="_blank" rel="noopener noreferrer">
                  {activity.repository_name}
                </a>
              </div>
              {metadata.title && (
                <div className="activity-details">
                  <a href={metadata.url} target="_blank" rel="noopener noreferrer">
                    #{metadata.number}: {metadata.title}
                  </a>
                </div>
              )}
              <div className="activity-date">{formatDate(activity.date)}</div>
            </div>
          </div>
        );

      default:
        return (
          <div className="activity-item" key={activity.id}>
            <div className="activity-icon">{getActivityIcon(activity.activity_type)}</div>
            <div className="activity-content">
              <div className="activity-header">{formatActivityType(activity.activity_type)}</div>
              {activity.repository_name && (
                <div className="activity-details">
                  <a href={activity.repository_url} target="_blank" rel="noopener noreferrer">
                    {activity.repository_name}
                  </a>
                </div>
              )}
              <div className="activity-date">{formatDate(activity.date)}</div>
            </div>
          </div>
        );
    }
  };

  if (error) {
    return (
      <div className="activity-timeline">
        <div className="error-message">Failed to load activities: {error}</div>
      </div>
    );
  }

  const groupedActivities = groupActivitiesByMonth();

  // Sort months chronologically (most recent first)
  const sortedMonths = Object.entries(groupedActivities).sort((a, b) => {
    const dateA = new Date(a[1][0].date); // Get date from first activity in month
    const dateB = new Date(b[1][0].date);
    return dateB - dateA; // Descending order (newest first)
  });

  return (
    <div className="activity-timeline">
      <div className="timeline-header">
        <h2 className="timeline-title">Contribution activity</h2>
      </div>

      {loading && activities.length === 0 ? (
        <div className="loading-message">Loading activities...</div>
      ) : activities.length === 0 ? (
        <div className="empty-message">
          <p>No activities yet.</p>
          <p>Go to the Platforms tab and click "Sync Activities" to fetch your contribution activity.</p>
        </div>
      ) : (
        <div className="timeline-content">
          {sortedMonths.map(([monthYear, monthActivities]) => (
            <div key={monthYear} className="month-group">
              <h3 className="month-header">{monthYear}</h3>
              <div className="activities-list">
                {monthActivities.map(activity => renderActivity(activity))}
              </div>
            </div>
          ))}

          {hasMore && (
            <div className="load-more-container">
              <button
                className="load-more-button"
                onClick={loadMore}
                disabled={loading}
              >
                {loading ? 'Loading...' : 'Show more activity'}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default ActivityTimeline;
