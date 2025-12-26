import { useState, useEffect } from 'react';
import apiClient from '../api/client';
import PlatformIcon from './PlatformIcon';
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
  const [syncingProfileId, setSyncingProfileId] = useState(null);
  const [showGiteaOAuthForm, setShowGiteaOAuthForm] = useState(false);
  const [giteaOAuthInstanceUrl, setGiteaOAuthInstanceUrl] = useState('');
  const [giteaOAuthError, setGiteaOAuthError] = useState(null);
  const [giteaInstances, setGiteaInstances] = useState([]);
  const [loadingGiteaInstances, setLoadingGiteaInstances] = useState(false);
  const [showGitlabOAuthForm, setShowGitlabOAuthForm] = useState(false);
  const [gitlabOAuthInstanceUrl, setGitlabOAuthInstanceUrl] = useState('');
  const [gitlabOAuthError, setGitlabOAuthError] = useState(null);
  const [gitlabInstances, setGitlabInstances] = useState([]);
  const [loadingGitlabInstances, setLoadingGitlabInstances] = useState(false);

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

  const handleConnectGiteaOAuth = async () => {
    console.log('🔐 [Gitea OAuth] Starting OAuth flow');

    if (!giteaOAuthInstanceUrl) {
      setGiteaOAuthError('Please select a Gitea instance');
      return;
    }

    try {
      setGiteaOAuthError(null);
      // Get OAuth authorization URL from backend (includes state token)
      const authUrl = await apiClient.startGiteaOAuth(giteaOAuthInstanceUrl);
      console.log(`🚀 [Gitea OAuth] Redirecting to: ${authUrl}`);
      window.location.href = authUrl;
    } catch (err) {
      console.error('❌ [Gitea OAuth] Failed to start OAuth flow:', err);
      setGiteaOAuthError(`Failed to start OAuth flow: ${err.message}`);
    }
  };

  const handleShowGiteaOAuth = async () => {
    setLoadingGiteaInstances(true);
    setGiteaOAuthError(null);
    setShowGiteaOAuthForm(true);

    try {
      // Fetch available Gitea OAuth instances
      const instances = await apiClient.listOAuthInstances('gitea');
      console.log('📋 Available Gitea instances:', instances);

      if (instances.length === 0) {
        setGiteaOAuthError('No Gitea OAuth apps configured. Please ask your administrator to configure a Gitea OAuth app in the admin panel.');
        setGiteaInstances([]);
      } else {
        setGiteaInstances(instances);
        // Auto-select the default instance or first instance
        const defaultInstance = instances.find(i => i.is_default) || instances[0];
        setGiteaOAuthInstanceUrl(defaultInstance.instance_url);

        // If only one instance, we can show a simpler UI
        if (instances.length === 1) {
          console.log(`✅ Auto-selected single Gitea instance: ${defaultInstance.instance_name}`);
        }
      }
    } catch (err) {
      console.error('❌ Failed to fetch Gitea instances:', err);
      setGiteaOAuthError(`Failed to load Gitea instances: ${err.message}`);
      setGiteaInstances([]);
    } finally {
      setLoadingGiteaInstances(false);
    }
  };

  const handleShowGitlabOAuth = async () => {
    setLoadingGitlabInstances(true);
    setGitlabOAuthError(null);
    setShowGitlabOAuthForm(true);

    try {
      // Fetch available GitLab OAuth instances
      const instances = await apiClient.listOAuthInstances('gitlab');
      console.log('📋 Available GitLab instances:', instances);

      if (instances.length === 0) {
        setGitlabOAuthError('No GitLab OAuth apps configured. Please ask your administrator to configure a GitLab OAuth app in the admin panel.');
        setGitlabInstances([]);
      } else {
        setGitlabInstances(instances);
        // Auto-select the default instance or first instance
        const defaultInstance = instances.find(i => i.is_default) || instances[0];
        setGitlabOAuthInstanceUrl(defaultInstance.instance_url);

        // If only one instance, we can show a simpler UI
        if (instances.length === 1) {
          console.log(`✅ Auto-selected single GitLab instance: ${defaultInstance.instance_name}`);
        }
      }
    } catch (err) {
      console.error('❌ Failed to fetch GitLab instances:', err);
      setGitlabOAuthError(`Failed to load GitLab instances: ${err.message}`);
      setGitlabInstances([]);
    } finally {
      setLoadingGitlabInstances(false);
    }
  };

  const handleConnectGitlabOAuth = async () => {
    console.log('🔐 [GitLab OAuth] Starting OAuth flow');

    if (!gitlabOAuthInstanceUrl) {
      setGitlabOAuthError('Please select a GitLab instance');
      return;
    }

    try {
      setGitlabOAuthError(null);
      const authUrl = await apiClient.startGitlabOAuth(gitlabOAuthInstanceUrl);
      console.log(`🚀 [GitLab OAuth] Redirecting to: ${authUrl}`);
      window.location.href = authUrl;
    } catch (err) {
      console.error('❌ [GitLab OAuth] Failed to start OAuth flow:', err);
      setGitlabOAuthError(`Failed to start OAuth flow: ${err.message}`);
    }
  };

  const handleConnectPAT = async (e) => {
    e.preventDefault();

    if (!patToken.trim()) {
      setPatError('Please enter a personal access token');
      return;
    }

    // Validate instance URL for Gitea and GitLab
    if ((selectedPlatform === 'gitea' || selectedPlatform === 'gitlab') && !instanceUrl.trim()) {
      setPatError(`Please enter your ${selectedPlatform === 'gitea' ? 'Gitea' : 'GitLab'} instance URL`);
      return;
    }

    try {
      setPatLoading(true);
      setPatError(null);
      await apiClient.connectPlatform(
        selectedPlatform,
        patToken,
        (selectedPlatform === 'gitea' || selectedPlatform === 'gitlab') ? instanceUrl : null
      );
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
      alert(`Sync (${yearText}) completed successfully! Heatmap and activities have been updated.`);
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
                    <PlatformIcon platform={platform.platform} size={16} />
                    <span>{platform.platform.charAt(0).toUpperCase() + platform.platform.slice(1)}</span>
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
                  <label className="sync-label" title="Syncs heatmap contributions and activity timeline together">
                    Heatmap & Activities:
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
                    title="Sync both heatmap contributions and activity timeline"
                  >
                    {syncingPlatformId === platform.id ? 'Syncing...' : 'Sync'}
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

        {!showPATForm && !showGiteaOAuthForm && !showGitlabOAuthForm ? (
          <div className="connect-buttons">
            <button className="btn btn-primary" onClick={handleConnectOAuth}>
              <PlatformIcon platform="github" size={16} />
              Connect GitHub with OAuth
            </button>
            <button className="btn btn-primary" onClick={handleShowGiteaOAuth}>
              <PlatformIcon platform="gitea" size={16} />
              Connect Gitea with OAuth
            </button>
            <button className="btn btn-primary" onClick={handleShowGitlabOAuth}>
              <PlatformIcon platform="gitlab" size={16} />
              Connect GitLab with OAuth
            </button>
            <button className="btn btn-secondary" onClick={() => {
              setSelectedPlatform('github');
              setShowPATForm(true);
            }}>
              Connect with Personal Access Token
            </button>
          </div>
        ) : showGitlabOAuthForm ? (
          <div className="gitlab-oauth-form">
            <h4>Connect GitLab with OAuth</h4>

            {loadingGitlabInstances ? (
              <p className="form-hint">Loading available GitLab instances...</p>
            ) : gitlabInstances.length === 0 ? (
              <>
                {gitlabOAuthError && <div className="error-message">{gitlabOAuthError}</div>}
                <p className="form-hint">
                  No GitLab OAuth applications are configured. Please ask your administrator to add a GitLab OAuth app in the admin panel at <code>/admin/oauth-apps</code>.
                </p>
              </>
            ) : (
              <>
                <p className="form-hint">
                  {gitlabInstances.length === 1
                    ? `Connecting to: ${gitlabInstances[0].instance_name}`
                    : 'Select which GitLab instance you want to connect to:'}
                </p>

                {gitlabOAuthError && <div className="error-message">{gitlabOAuthError}</div>}

                {gitlabInstances.length > 1 && (
                  <div className="instance-selector">
                    <label htmlFor="gitlab-instance-select">GitLab Instance:</label>
                    <select
                      id="gitlab-instance-select"
                      value={gitlabOAuthInstanceUrl}
                      onChange={(e) => setGitlabOAuthInstanceUrl(e.target.value)}
                      className="instance-select"
                    >
                      {gitlabInstances.map((instance) => (
                        <option key={instance.instance_url} value={instance.instance_url}>
                          {instance.instance_name} ({instance.instance_url || 'gitlab.com'})
                          {instance.is_default ? ' [Default]' : ''}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
              </>
            )}

            <div className="pat-form-actions">
              {gitlabInstances.length > 0 && (
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={handleConnectGitlabOAuth}
                  disabled={loadingGitlabInstances}
                >
                  Connect
                </button>
              )}
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  setShowGitlabOAuthForm(false);
                  setGitlabOAuthInstanceUrl('');
                  setGitlabOAuthError(null);
                  setGitlabInstances([]);
                }}
                disabled={loadingGitlabInstances}
              >
                Cancel
              </button>
            </div>
          </div>
        ) : showGiteaOAuthForm ? (
          <div className="gitea-oauth-form">
            <h4>Connect Gitea with OAuth</h4>

            {loadingGiteaInstances ? (
              <p className="form-hint">Loading available Gitea instances...</p>
            ) : giteaInstances.length === 0 ? (
              <>
                {giteaOAuthError && <div className="error-message">{giteaOAuthError}</div>}
                <p className="form-hint">
                  No Gitea OAuth applications are configured. Please ask your administrator to add a Gitea OAuth app in the admin panel at <code>/admin/oauth-apps</code>.
                </p>
              </>
            ) : (
              <>
                <p className="form-hint">
                  {giteaInstances.length === 1
                    ? `Connecting to: ${giteaInstances[0].instance_name}`
                    : 'Select which Gitea instance you want to connect to:'}
                </p>

                {giteaOAuthError && <div className="error-message">{giteaOAuthError}</div>}

                {giteaInstances.length > 1 && (
                  <div className="instance-selector">
                    <label htmlFor="gitea-instance-select">Gitea Instance:</label>
                    <select
                      id="gitea-instance-select"
                      value={giteaOAuthInstanceUrl}
                      onChange={(e) => setGiteaOAuthInstanceUrl(e.target.value)}
                      className="instance-select"
                    >
                      {giteaInstances.map((instance) => (
                        <option key={instance.instance_url} value={instance.instance_url}>
                          {instance.instance_name} ({instance.instance_url})
                          {instance.is_default ? ' [Default]' : ''}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
              </>
            )}

            <div className="pat-form-actions">
              {giteaInstances.length > 0 && (
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={handleConnectGiteaOAuth}
                  disabled={loadingGiteaInstances}
                >
                  Connect
                </button>
              )}
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  setShowGiteaOAuthForm(false);
                  setGiteaOAuthInstanceUrl('');
                  setGiteaOAuthError(null);
                  setGiteaInstances([]);
                }}
                disabled={loadingGiteaInstances}
              >
                Cancel
              </button>
            </div>
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
                <option value="gitlab">GitLab</option>
              </select>
            </div>

            {(selectedPlatform === 'gitea' || selectedPlatform === 'gitlab') && (
              <div className="instance-url-input">
                <label htmlFor="instance-url">Instance URL:</label>
                <input
                  id="instance-url"
                  type="url"
                  value={instanceUrl}
                  onChange={(e) => setInstanceUrl(e.target.value)}
                  placeholder={selectedPlatform === 'gitea' ? "https://gitea.example.com" : "https://gitlab.com"}
                  className="instance-url"
                  disabled={patLoading}
                  required
                />
                <p className="input-hint">
                  {selectedPlatform === 'gitea'
                    ? "Enter the full URL of your Gitea instance"
                    : "Enter the full URL of your GitLab instance (e.g., https://gitlab.com for GitLab.com)"}
                </p>
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
              ) : selectedPlatform === 'gitlab' ? (
                <>
                  Create a GitLab Personal Access Token with these scopes:
                  <br />
                  <code>read_user</code>, <code>read_api</code>, <code>read_repository</code>
                  <br />
                  Go to your GitLab instance → Preferences → Access Tokens → Add new token
                  <br />
                  For gitlab.com: <a href="https://gitlab.com/-/user_settings/personal_access_tokens" target="_blank" rel="noopener noreferrer">Create token →</a>
                </>
              ) : null}
            </p>

            {patError && <div className="error-message">{patError}</div>}

            <input
              type="password"
              value={patToken}
              onChange={(e) => setPatToken(e.target.value)}
              placeholder={
                selectedPlatform === 'github' ? 'ghp_xxxxxxxxxxxx' :
                selectedPlatform === 'gitea' ? 'Your Gitea token' :
                selectedPlatform === 'gitlab' ? 'glpat-xxxxxxxxxxxx' :
                'Token'
              }
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
