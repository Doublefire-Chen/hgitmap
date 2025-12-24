import { useState, useEffect } from 'react';
import apiClient from '../api/client';
import './SyncSettings.css';

function SyncSettings() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncingPlatform, setSyncingPlatform] = useState(null); // Track which platform is syncing
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(null);

  const [settings, setSettings] = useState({
    auto_generation_enabled: false,
    update_interval_minutes: 60,
    date_range_days: 365,
    include_private_contributions: true,
  });

  const [syncStatus, setSyncStatus] = useState(null);
  const [platforms, setPlatforms] = useState([]);

  useEffect(() => {
    loadSettings();
    loadSyncStatus();
    loadPlatforms();
  }, []);

  const loadPlatforms = async () => {
    try {
      const platformData = await apiClient.listPlatforms();
      setPlatforms(platformData);
    } catch (err) {
      console.error('Failed to load platforms:', err);
    }
  };

  const loadSettings = async () => {
    try {
      setLoading(true);
      const data = await apiClient.getGenerationSettings();
      setSettings({
        auto_generation_enabled: data.auto_generation_enabled,
        update_interval_minutes: data.update_interval_minutes,
        date_range_days: data.date_range_days,
        include_private_contributions: data.include_private_contributions,
      });
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const loadSyncStatus = async () => {
    try {
      const status = await apiClient.getSyncStatus();
      setSyncStatus(status);
    } catch (err) {
      console.error('Failed to load sync status:', err);
    }
  };

  const handleSave = async () => {
    // Validate that if auto sync is being enabled, at least one platform has at least one sync type enabled
    if (settings.auto_generation_enabled) {
      const hasAnySyncEnabled = platforms.some(p =>
        p.sync_profile || p.sync_contributions || p.sync_activities
      );

      if (!hasAnySyncEnabled) {
        setError('Cannot enable automatic sync: At least one platform must have at least one sync type enabled (Profile, Heatmap, or Activities)');
        return;
      }
    }

    try {
      setSaving(true);
      setError(null);
      setSuccess(null);

      await apiClient.updateGenerationSettings(settings);

      setSuccess('Settings saved successfully!');
      setTimeout(() => setSuccess(null), 3000);

      // Reload sync status to see updated next sync time
      await loadSyncStatus();
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleManualSync = async () => {
    try {
      setSyncing(true);
      setError(null);
      setSuccess(null);

      const result = await apiClient.triggerSync();

      if (result.success) {
        setSuccess(result.message);
      } else {
        setError(result.message);
      }

      // Reload sync status
      await loadSyncStatus();

      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err.message);
    } finally {
      setSyncing(false);
    }
  };

  const handlePlatformSyncPreferenceChange = async (platformId, preferences) => {
    // Validate that at least one sync type is enabled
    if (!preferences.sync_profile && !preferences.sync_contributions && !preferences.sync_activities) {
      setError('At least one sync type must be enabled (Profile, Heatmap, or Activities)');
      setTimeout(() => setError(null), 5000);
      return;
    }

    try {
      await apiClient.updatePlatformSyncPreferences(platformId, preferences);

      // Update local state
      setPlatforms(platforms.map(p =>
        p.id === platformId
          ? { ...p, ...preferences }
          : p
      ));

      setSuccess('Sync preferences updated successfully!');
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err.message);
      // Reload platforms to revert changes
      await loadPlatforms();
    }
  };

  const handlePlatformActivitySync = async (platformId, allYears = false) => {
    try {
      setSyncingPlatform(platformId);
      setError(null);
      setSuccess(null);

      await apiClient.syncActivities(allYears, null, platformId);

      setSuccess(`Activities synced successfully for this platform!`);
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(err.message || 'Failed to sync activities');
    } finally {
      setSyncingPlatform(null);
    }
  };

  const formatDateTime = (dateStr) => {
    if (!dateStr) return 'Never';
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  if (loading) {
    return <div className="sync-settings-container"><div className="loading">Loading settings...</div></div>;
  }

  return (
    <div className="sync-settings-container">
      <div className="settings-header">
        <h1>Sync Settings</h1>
        <p className="subtitle">Configure automatic syncing of your git platform data</p>
      </div>

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">{success}</div>}

      {/* Sync Status */}
      {syncStatus && (
        <section className="status-section">
          <h2>Sync Status</h2>
          <div className="status-grid">
            <div className="status-item">
              <label>Auto Sync</label>
              <span className={syncStatus.auto_sync_enabled ? 'status-enabled' : 'status-disabled'}>
                {syncStatus.auto_sync_enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div className="status-item">
              <label>Last Sync</label>
              <span>{formatDateTime(syncStatus.last_sync)}</span>
            </div>
            <div className="status-item">
              <label>Next Sync</label>
              <span>{formatDateTime(syncStatus.next_sync)}</span>
            </div>
            <div className="status-item">
              <label>Sync Interval</label>
              <span>{syncStatus.update_interval_minutes} minutes</span>
            </div>
          </div>

          {syncStatus.platform_accounts && syncStatus.platform_accounts.length > 0 && (
            <div className="platforms-status">
              <h3>Platform Accounts</h3>
              <div className="platforms-list">
                {syncStatus.platform_accounts.map((platform, idx) => (
                  <div key={idx} className="platform-item">
                    <div className="platform-name">
                      <strong>{platform.platform}</strong>: {platform.username}
                    </div>
                    <div className="platform-sync">
                      Last synced: {formatDateTime(platform.last_synced)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <button
            className="btn-primary sync-button"
            onClick={handleManualSync}
            disabled={syncing}
          >
            {syncing ? 'Syncing...' : 'Sync Now'}
          </button>
        </section>
      )}

      {/* Per-Platform Sync Preferences */}
      {platforms && platforms.length > 0 && (
        <section className="settings-section">
          <h2>Platform Sync Preferences</h2>
          <p className="subtitle">Control what data to sync for each connected platform</p>

          {platforms.map((platform) => (
            <div key={platform.id} className="platform-sync-preferences">
              <div className="platform-header">
                <div className="platform-info">
                  <strong>{platform.platform}</strong>: {platform.platform_username}
                </div>
              </div>

              <div className="sync-checkboxes">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={platform.sync_profile}
                    onChange={(e) => handlePlatformSyncPreferenceChange(platform.id, {
                      sync_profile: e.target.checked,
                      sync_contributions: platform.sync_contributions,
                      sync_activities: platform.sync_activities,
                    })}
                  />
                  <span>Profile</span>
                </label>

                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={platform.sync_contributions}
                    onChange={(e) => handlePlatformSyncPreferenceChange(platform.id, {
                      sync_profile: platform.sync_profile,
                      sync_contributions: e.target.checked,
                      sync_activities: platform.sync_activities,
                    })}
                  />
                  <span>Heatmap</span>
                </label>

                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={platform.sync_activities}
                    onChange={(e) => handlePlatformSyncPreferenceChange(platform.id, {
                      sync_profile: platform.sync_profile,
                      sync_contributions: platform.sync_contributions,
                      sync_activities: e.target.checked,
                    })}
                  />
                  <span>Activities</span>
                </label>
              </div>

              <div className="platform-actions">
                <button
                  className="btn-secondary"
                  onClick={() => handlePlatformActivitySync(platform.id, false)}
                  disabled={syncingPlatform === platform.id}
                  title="Sync current year activities"
                >
                  {syncingPlatform === platform.id ? 'Syncing...' : 'Sync Activities (Current Year)'}
                </button>
                <button
                  className="btn-secondary"
                  onClick={() => handlePlatformActivitySync(platform.id, true)}
                  disabled={syncingPlatform === platform.id}
                  title="Sync all years activities"
                >
                  {syncingPlatform === platform.id ? 'Syncing...' : 'Sync Activities (All Years)'}
                </button>
              </div>

              <p className="hint-text">
                Select which types of data to sync from this platform. Use the buttons above to manually sync activities for this platform only.
              </p>
            </div>
          ))}
        </section>
      )}

      {/* Settings Form */}
      <section className="settings-section">
        <h2>Auto Sync Configuration</h2>

        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.auto_generation_enabled}
              onChange={(e) => setSettings({ ...settings, auto_generation_enabled: e.target.checked })}
            />
            <span>Enable automatic sync</span>
          </label>
          <p className="hint-text">
            When enabled, your git platform data will be automatically synced based on the interval below
          </p>
        </div>

        {settings.auto_generation_enabled && (
          <div className="form-group">
            <label>Sync Interval (minutes)</label>
            <select
              value={settings.update_interval_minutes}
              onChange={(e) => setSettings({ ...settings, update_interval_minutes: parseInt(e.target.value) })}
            >
              <option value={15}>Every 15 minutes</option>
              <option value={30}>Every 30 minutes</option>
              <option value={60}>Every hour</option>
              <option value={120}>Every 2 hours</option>
              <option value={180}>Every 3 hours</option>
              <option value={360}>Every 6 hours</option>
              <option value={720}>Every 12 hours</option>
              <option value={1440}>Once a day</option>
            </select>
            <p className="hint-text">
              How often to automatically fetch new contribution data from your connected platforms
            </p>
          </div>
        )}

        <div className="form-group">
          <label>Date Range (days)</label>
          <input
            type="number"
            value={settings.date_range_days}
            onChange={(e) => setSettings({ ...settings, date_range_days: parseInt(e.target.value) })}
            min="30"
            max="730"
          />
          <p className="hint-text">
            Number of days to include in your heatmap (30-730 days)
          </p>
        </div>

        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.include_private_contributions}
              onChange={(e) => setSettings({ ...settings, include_private_contributions: e.target.checked })}
            />
            <span>Include private repository contributions</span>
          </label>
          <p className="hint-text">
            Show contributions from private repositories in your heatmap
          </p>
        </div>

        <div className="form-actions">
          <button
            className="btn-primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </section>
    </div>
  );
}

export default SyncSettings;
