import { useState, useEffect } from 'react';
import apiClient from '../api/client';
import './PlatformConnector.css';

function PlatformConnector() {
  const [platforms, setPlatforms] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showPATForm, setShowPATForm] = useState(false);
  const [selectedPlatform, setSelectedPlatform] = useState('github'); // 'github' or 'gitea'
  const [patToken, setPatToken] = useState('');
  const [instanceUrl, setInstanceUrl] = useState(''); // For Gitea/GitLab custom instances
  const [patError, setPatError] = useState(null);
  const [patLoading, setPatLoading] = useState(false);
  const [syncingPlatformId, setSyncingPlatformId] = useState(null);
  const [selectedSyncYear, setSelectedSyncYear] = useState({});
  const [syncingActivitiesPlatformId, setSyncingActivitiesPlatformId] = useState(null);
  const [selectedActivityYear, setSelectedActivityYear] = useState({});
  const [syncingProfileId, setSyncingProfileId] = useState(null);

  useEffect(() => {
    loadPlatforms();
  }, []);

  const loadPlatforms = async () => {
    try {
      setLoading(true);
      const data = await apiClient.listPlatforms();
      setPlatforms(data);
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleConnectOAuth = async () => {
    console.log('🔐 [OAuth] Starting OAuth flow');

    try {
      // Get OAuth authorization URL from backend (includes state token)
      const authUrl = await apiClient.startGithubOAuth();
      console.log(`🚀 [OAuth] Redirecting to: ${authUrl}`);
      window.location.href = authUrl;
    } catch (err) {
      console.error('❌ [OAuth] Failed to start OAuth flow:', err);
      setError(`Failed to start OAuth flow: ${err.message}`);
    }
  };

  const handleConnectPAT = async (e) => {
    e.preventDefault();

    if (!patToken.trim()) {
      setPatError('Please enter a personal access token');
      return;
    }

    // Validate instance URL for Gitea
    if (selectedPlatform === 'gitea' && !instanceUrl.trim()) {
      setPatError('Please enter your Gitea instance URL');
      return;
    }

    try {
      setPatLoading(true);
      setPatError(null);
      await apiClient.connectPlatform(selectedPlatform, patToken, selectedPlatform === 'gitea' ? instanceUrl : null);
      setPatToken('');
      setInstanceUrl('');
      setShowPATForm(false);
      await loadPlatforms();
    } catch (err) {
      setPatError(err.message);
    } finally {
      setPatLoading(false);
    }
  };

  const handleDisconnect = async (platformId) => {
    if (!confirm('Are you sure you want to disconnect this platform?')) {
      return;
    }

    try {
      await apiClient.disconnectPlatform(platformId);
      await loadPlatforms();
    } catch (err) {
      setError(err.message);
    }
  };

  const handleSync = async (platformId) => {
    try {
      setSyncingPlatformId(platformId);
      const syncOption = selectedSyncYear[platformId] || 'current';

      let allYears = false;
      let year = null;

      if (syncOption === 'all') {
        allYears = true;
      } else if (syncOption !== 'current') {
        year = parseInt(syncOption);
      }

      await apiClient.syncPlatform(platformId, allYears, year);

      const yearText = allYears ? 'all years' : year ? `year ${year}` : 'current year';
      alert(`Sync (${yearText}) completed successfully! Profile data and contributions have been updated.`);
      await loadPlatforms();

      // Trigger a custom event to refresh profile display
      window.dispatchEvent(new CustomEvent('platformSynced'));
    } catch (err) {
      alert(`Sync failed: ${err.message}`);
    } finally {
      setSyncingPlatformId(null);
    }
  };

  const handleSyncProfile = async (platformId) => {
    try {
      setSyncingProfileId(platformId);

      // Sync current year only to minimize data transfer - this will update profile data
      await apiClient.syncPlatform(platformId, false, null);

      alert('Profile synced successfully! Avatar, bio, and other profile details have been updated.');
      await loadPlatforms();

      // Trigger a custom event to refresh profile display
      window.dispatchEvent(new CustomEvent('platformSynced'));
    } catch (err) {
      alert(`Profile sync failed: ${err.message}`);
    } finally {
      setSyncingProfileId(null);
    }
  };

  const handleSyncActivities = async (platformId) => {
    try {
      setSyncingActivitiesPlatformId(platformId);
      const syncOption = selectedActivityYear[platformId] || 'current';

      let allYears = false;
      let year = null;

      if (syncOption === 'all') {
        allYears = true;
      } else if (syncOption !== 'current') {
        year = parseInt(syncOption);
      }

      await apiClient.syncActivities(allYears, year);

      const yearText = allYears ? 'all years' : year ? `year ${year}` : 'current year';
      alert(`Activity sync (${yearText}) completed successfully! Go to Overview tab to see the results.`);
    } catch (err) {
      alert(`Activity sync failed: ${err.message}`);
    } finally {
      setSyncingActivitiesPlatformId(null);
    }
  };

  if (loading) {
    return <div className="platform-connector loading">Loading platforms...</div>;
  }

  return (
    <div className="platform-connector">
      <div className="platform-connector-header">
        <h2>Connected Platforms</h2>
        {platforms.length === 0 && (
          <p className="no-platforms-message">
            Connect your GitHub account to start tracking contributions
          </p>
        )}
      </div>

      {error && <div className="error-message">{error}</div>}

      {/* Connected Platforms List */}
      {platforms.length > 0 && (
        <div className="platforms-list">
          {platforms.map((platform) => (
            <div key={platform.id} className="platform-card">
              <div className="platform-info">
                <div className="platform-details">
                  <h3>{platform.platform_username}</h3>
                  <span className="platform-name">
                    {platform.platform.charAt(0).toUpperCase() + platform.platform.slice(1)}
                  </span>
                  {platform.last_synced_at && (
                    <span className="last-sync">
                      Last synced: {new Date(platform.last_synced_at).toLocaleString()}
                    </span>
                  )}
                </div>
              </div>

              <div className="platform-sync-section">
                <div className="sync-row">
                  <label className="sync-label">Profile:</label>
                  <button
                    className="btn btn-secondary btn-sm profile-sync-btn"
                    onClick={() => handleSyncProfile(platform.id)}
                    disabled={syncingProfileId === platform.id}
                    title="Refresh profile information (avatar, bio, location, company)"
                  >
                    {syncingProfileId === platform.id ? 'Syncing...' : 'Sync Profile'}
                  </button>
                </div>

                <div className="sync-row">
                  <label className="sync-label" title="Syncs contributions">
                    Heatmap:
                  </label>
                  <select
                    className="sync-year-selector"
                    value={selectedSyncYear[platform.id] || 'current'}
                    onChange={(e) => setSelectedSyncYear({ ...selectedSyncYear, [platform.id]: e.target.value })}
                    disabled={syncingPlatformId === platform.id}
                  >
                    <option value="current">Current year ({new Date().getFullYear()})</option>
                    {Array.from({ length: new Date().getFullYear() - 2019 }, (_, i) => {
                      const year = new Date().getFullYear() - 1 - i;
                      return <option key={year} value={year}>{year}</option>;
                    })}
                    <option value="all">All years (2020-{new Date().getFullYear()})</option>
                  </select>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => handleSync(platform.id)}
                    disabled={syncingPlatformId === platform.id}
                    title="Sync contribution data"
                  >
                    {syncingPlatformId === platform.id ? 'Syncing...' : 'Sync'}
                  </button>
                </div>

                <div className="sync-row">
                  <label className="sync-label">Activities:</label>
                  <select
                    className="sync-year-selector"
                    value={selectedActivityYear[platform.id] || 'current'}
                    onChange={(e) => setSelectedActivityYear({ ...selectedActivityYear, [platform.id]: e.target.value })}
                    disabled={syncingActivitiesPlatformId === platform.id}
                  >
                    <option value="current">Current year ({new Date().getFullYear()})</option>
                    {Array.from({ length: new Date().getFullYear() - 2019 }, (_, i) => {
                      const year = new Date().getFullYear() - 1 - i;
                      return <option key={year} value={year}>{year}</option>;
                    })}
                    <option value="all">All years (2020-{new Date().getFullYear()})</option>
                  </select>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => handleSyncActivities(platform.id)}
                    disabled={syncingActivitiesPlatformId === platform.id}
                  >
                    {syncingActivitiesPlatformId === platform.id ? 'Syncing...' : 'Sync'}
                  </button>
                </div>

                <button
                  className="btn btn-danger btn-sm remove-btn"
                  onClick={() => handleDisconnect(platform.id)}
                >
                  Remove
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Connect New Platform */}
      <div className="connect-platform-section">
        <h3>Connect Platform</h3>

        {!showPATForm ? (
          <div className="connect-buttons">
            <button className="btn btn-primary" onClick={handleConnectOAuth}>
              Connect GitHub with OAuth
            </button>
            <button className="btn btn-secondary" onClick={() => {
              setSelectedPlatform('github');
              setShowPATForm(true);
            }}>
              Connect with Personal Access Token
            </button>
          </div>
        ) : (
          <form onSubmit={handleConnectPAT} className="pat-form">
            <div className="platform-selector">
              <label htmlFor="platform-select">Platform:</label>
              <select
                id="platform-select"
                value={selectedPlatform}
                onChange={(e) => setSelectedPlatform(e.target.value)}
                disabled={patLoading}
                className="platform-select"
              >
                <option value="github">GitHub</option>
                <option value="gitea">Gitea</option>
              </select>
            </div>

            {selectedPlatform === 'gitea' && (
              <div className="instance-url-input">
                <label htmlFor="instance-url">Instance URL:</label>
                <input
                  id="instance-url"
                  type="url"
                  value={instanceUrl}
                  onChange={(e) => setInstanceUrl(e.target.value)}
                  placeholder="https://gitea.example.com"
                  className="instance-url"
                  disabled={patLoading}
                />
                <p className="input-hint">Enter the full URL of your Gitea instance</p>
              </div>
            )}

            <p className="pat-instructions">
              {selectedPlatform === 'github' ? (
                <>
                  Create a GitHub Personal Access Token with <code>repo</code> and <code>read:user</code> scopes.
                  <br />
                  <a
                    href="https://github.com/settings/tokens/new?scopes=repo,read:user&description=hgitmap"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    Create token →
                  </a>
                </>
              ) : selectedPlatform === 'gitea' ? (
                <>
                  Create a Gitea Personal Access Token with <code>read:repository</code>, <code>read:user</code>, and <code>read:organization</code> scopes.
                  <br />
                  Go to your Gitea instance → Settings → Applications → Generate New Token
                </>
              ) : null}
            </p>

            {patError && <div className="error-message">{patError}</div>}

            <input
              type="password"
              value={patToken}
              onChange={(e) => setPatToken(e.target.value)}
              placeholder={selectedPlatform === 'github' ? 'ghp_xxxxxxxxxxxx' : selectedPlatform === 'gitea' ? 'Your Gitea token' : 'Token'}
              className="pat-input"
              disabled={patLoading}
            />

            <div className="pat-form-actions">
              <button type="submit" className="btn btn-primary" disabled={patLoading}>
                {patLoading ? 'Connecting...' : 'Connect'}
              </button>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  setShowPATForm(false);
                  setPatToken('');
                  setInstanceUrl('');
                  setPatError(null);
                  setSelectedPlatform('github');
                }}
                disabled={patLoading}
              >
                Cancel
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

export default PlatformConnector;
