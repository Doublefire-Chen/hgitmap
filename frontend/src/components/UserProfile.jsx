import { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { useTheme } from '../context/ThemeContext';
import apiClient from '../api/client';
import './UserProfile.css';
import GitHubLogoLight from '../assets/github-light.svg';
import GitHubLogoDark from '../assets/github-dark.svg';

const UserProfile = () => {
  const { user } = useAuth();
  const { theme } = useTheme();
  const [stats, setStats] = useState(null);
  const [platforms, setPlatforms] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch contribution stats
        const statsData = await apiClient.getContributionStats();
        setStats(statsData);

        // Fetch connected platforms
        const platformsData = await apiClient.getPlatforms();
        setPlatforms(platformsData);
      } catch (error) {
        console.error('Failed to fetch user data:', error);
      }
    };

    fetchData();

    // Listen for platform sync events to refresh profile data
    const handlePlatformSynced = () => {
      console.log('Platform synced, refreshing profile data...');
      fetchData();
    };

    window.addEventListener('platformSynced', handlePlatformSynced);

    return () => {
      window.removeEventListener('platformSynced', handlePlatformSynced);
    };
  }, []);

  // Generate fallback avatar URL (using UI Avatars service)
  const getFallbackAvatarUrl = (username) => {
    return `https://ui-avatars.com/api/?name=${encodeURIComponent(username)}&size=260&background=random&bold=true`;
  };

  // Get platform logo
  const getPlatformLogo = (platformType) => {
    switch (platformType) {
      case 'github':
        return theme === 'dark' ? GitHubLogoDark : GitHubLogoLight;
      default:
        return null;
    }
  };

  // Get platform name
  const getPlatformName = (platformType) => {
    switch (platformType) {
      case 'github':
        return 'GitHub';
      case 'gitlab':
        return 'GitLab';
      case 'gitea':
        return 'Gitea';
      default:
        return platformType;
    }
  };

  return (
    <div className="user-profile">
      {/* Overall Stats Section */}
      {stats && (
        <div className="heatmap-stats">
          <div className="stat-card">
            <div className="stat-label">Total</div>
            <div className="stat-value">{stats.total_contributions?.toLocaleString() || 0} contributions</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Current streak</div>
            <div className="stat-value">{stats.current_streak || 0} days</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Longest streak</div>
            <div className="stat-value">{stats.longest_streak || 0} days</div>
          </div>

          <div className="stat-card">
            <div className="stat-label">Platforms</div>
            <div className="stat-value">{platforms.length} connected</div>
          </div>
        </div>
      )}

      {/* Platform Profiles Section */}
      {platforms && platforms.length > 0 && (
        <div className="platform-profiles">
          {platforms.map((platform) => (
            <div key={platform.id} className="platform-profile-card">
              <div className="platform-header">
                {getPlatformLogo(platform.platform) && (
                  <img
                    src={getPlatformLogo(platform.platform)}
                    alt={getPlatformName(platform.platform)}
                    className="platform-logo"
                  />
                )}
                <span className="platform-type-name">
                  {getPlatformName(platform.platform).toUpperCase()}
                </span>
              </div>

              <div className="platform-avatar">
                <img
                  src={platform.avatar_url || getFallbackAvatarUrl(platform.platform_username)}
                  alt={platform.platform_username}
                  className="platform-avatar-img"
                  onError={(e) => {
                    e.target.src = getFallbackAvatarUrl(platform.platform_username);
                  }}
                />
              </div>

              <div className="platform-info">
                <div className="vcard-names">
                  <h1 className="profile-name-heading">
                    <span className="platform-display-name">
                      {platform.display_name || platform.platform_username}
                    </span>
                    <span className="platform-username">
                      {platform.platform_username}
                    </span>
                  </h1>
                </div>

                {platform.bio && (
                  <div className="platform-bio">{platform.bio}</div>
                )}

                {(platform.location || platform.company) && (
                  <div className="platform-details">
                    {platform.location && (
                      <div className="detail-item">
                        <span className="detail-icon">📍</span>
                        <span className="detail-text">{platform.location}</span>
                      </div>
                    )}

                    {platform.company && (
                      <div className="detail-item">
                        <span className="detail-icon">🏢</span>
                        <span className="detail-text">{platform.company}</span>
                      </div>
                    )}
                  </div>
                )}

                {((platform.followers_count !== null && platform.followers_count !== undefined) ||
                  (platform.following_count !== null && platform.following_count !== undefined)) && (
                  <div className="platform-followers">
                    <svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" className="followers-icon">
                      <path d="M2 5.5a3.5 3.5 0 1 1 5.898 2.549 5.508 5.508 0 0 1 3.034 4.084.75.75 0 1 1-1.482.235 4 4 0 0 0-7.9 0 .75.75 0 0 1-1.482-.236A5.507 5.507 0 0 1 3.102 8.05 3.493 3.493 0 0 1 2 5.5ZM11 4a3.001 3.001 0 0 1 2.22 5.018 5.01 5.01 0 0 1 2.56 3.012.749.749 0 0 1-.885.954.752.752 0 0 1-.549-.514 3.507 3.507 0 0 0-2.522-2.372.75.75 0 0 1-.574-.73v-.352a.75.75 0 0 1 .416-.672A1.5 1.5 0 0 0 11 5.5.75.75 0 0 1 11 4Zm-5.5-.5a2 2 0 1 0-.001 3.999A2 2 0 0 0 5.5 3.5Z" fill="currentColor"></path>
                    </svg>
                    {(platform.followers_count !== null && platform.followers_count !== undefined) && (
                      <>
                        <span className="follower-count">{platform.followers_count}</span>
                        <span className="follower-label">followers</span>
                      </>
                    )}
                    {(platform.followers_count !== null && platform.followers_count !== undefined) &&
                     (platform.following_count !== null && platform.following_count !== undefined) && (
                      <span className="follower-separator">·</span>
                    )}
                    {(platform.following_count !== null && platform.following_count !== undefined) && (
                      <>
                        <span className="follower-count">{platform.following_count}</span>
                        <span className="follower-label">following</span>
                      </>
                    )}
                  </div>
                )}

                {platform.profile_url && (
                  <a
                    href={platform.profile_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="platform-profile-link"
                  >
                    View Profile →
                  </a>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Empty State */}
      {(!platforms || platforms.length === 0) && (
        <div className="empty-state">
          <p className="empty-message">No platforms connected yet</p>
          <p className="empty-hint">Connect a platform to see your profile</p>
        </div>
      )}
    </div>
  );
};

export default UserProfile;
