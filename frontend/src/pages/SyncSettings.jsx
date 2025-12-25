import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import apiClient from '../api/client';
import './SyncSettings.css';

function SyncSettings() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
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

  const handlePlatformSyncPreferenceChange = async (platformId, preferences) => {
    // Validate that at least one sync type is enabled
    if (!preferences.sync_profile && !preferences.sync_contributions) {
      setError('At least one sync type must be enabled (Profile or Heatmap+Activities)');
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
        <div className="breadcrumb">
          <Link to="/" className="breadcrumb-link">Dashboard</Link>
          <span className="breadcrumb-separator">/</span>
          <span className="breadcrumb-current">Scheduled Sync Settings</span>
        </div>
        <h1>Scheduled Sync Settings</h1>
        <p className="subtitle">Configure automatic background syncing of your git platform data</p>
        <p className="hint-text">
          💡 For manual syncing, go to <Link to="/" className="inline-link">Dashboard → Platforms</Link>
        </p>
      </div>

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">{success}</div>}

      {/* Sync Status - Read Only */}
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
              <span>{syncStatus.next_scheduled_generation_at ? formatDateTime(syncStatus.next_scheduled_generation_at) : formatDateTime(syncStatus.next_sync)}</span>
            </div>
            <div className="status-item">
              <label>Sync Interval</label>
              <span>{syncStatus.update_interval_minutes} minutes</span>
            </div>
          </div>

          {syncStatus.platform_accounts && syncStatus.platform_accounts.length > 0 && (
            <div className="platforms-status">
              <h3>Connected Platforms</h3>
              <p className="hint-text">These platforms will be included in automatic syncing</p>
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
        </section>
      )}

      {/* Per-Platform Sync Preferences */}
      {platforms && platforms.length > 0 && (
        <section className="settings-section">
          <h2>What to Sync</h2>
          <p className="subtitle">Configure which data types to sync for each platform during automatic syncing</p>

          {platforms.map((platform) => (
            <div key={platform.id} className="platform-sync-preferences">
              <div className="platform-info">
                <strong>{platform.platform}</strong>: {platform.platform_username}
                {platform.last_synced_at && (
                  <span className="last-synced">
                    {' '}• Last synced: {formatDateTime(platform.last_synced_at)}
                  </span>
                )}
              </div>

              <div className="sync-checkboxes">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={platform.sync_profile}
                    onChange={(e) => handlePlatformSyncPreferenceChange(platform.id, {
                      sync_profile: e.target.checked,
                      sync_contributions: platform.sync_contributions,
                    })}
                  />
                  <span>Profile</span>
                  <span className="hint-inline">(avatar, bio, followers)</span>
                </label>

                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={platform.sync_contributions}
                    onChange={(e) => handlePlatformSyncPreferenceChange(platform.id, {
                      sync_profile: platform.sync_profile,
                      sync_contributions: e.target.checked,
                    })}
                  />
                  <span>Heatmap + Activities</span>
                  <span className="hint-inline">(contributions and timeline events)</span>
                </label>
              </div>
            </div>
          ))}
        </section>
      )}

      {/* Auto Sync Configuration Form */}
      <section className="settings-section">
        <h2>When to Sync</h2>
        <p className="subtitle">Configure automatic background sync schedule</p>

        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.auto_generation_enabled}
              onChange={(e) => setSettings({ ...settings, auto_generation_enabled: e.target.checked })}
            />
            <span>Enable automatic background sync</span>
          </label>
          <p className="hint-text">
            When enabled, your git platform data will be automatically synced in the background based on the interval below
          </p>
        </div>

        {settings.auto_generation_enabled && (
          <div className="form-group">
            <label>Sync Interval</label>
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
              How often to automatically fetch new data from your connected platforms (syncs current year only)
            </p>
          </div>
        )}

        <div className="form-group">
          <label>Heatmap Date Range</label>
          <input
            type="number"
            value={settings.date_range_days}
            onChange={(e) => setSettings({ ...settings, date_range_days: parseInt(e.target.value) })}
            min="30"
            max="730"
          />
          <p className="hint-text">
            Number of days to include in your heatmap visualization (30-730 days)
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
          <Link to="/" className="btn-secondary">
            Back to Dashboard
          </Link>
        </div>
      </section>
    </div>
  );
}

export default SyncSettings;
