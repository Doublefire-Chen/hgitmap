import { useState, useEffect } from 'react';
import { useToast } from '../context/ToastContext';
import apiClient from '../api/client';
import ThemeToggle from './ThemeToggle';
import './UserSettings.css';

export default function UserSettings() {
  const { error: showError, success: showSuccess } = useToast();
  const [settings, setSettings] = useState({
    show_private_contributions: true,
    hide_private_repo_names: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showRequirementWarning, setShowRequirementWarning] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const data = await apiClient.getSettings();
      setSettings({
        show_private_contributions: data.show_private_contributions,
        hide_private_repo_names: data.hide_private_repo_names,
      });
    } catch (err) {
      showError(err.message || 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (e) => {
    e.preventDefault();

    try {
      setSaving(true);
      await apiClient.updateSettings(settings);
      showSuccess('Settings saved successfully!');
    } catch (err) {
      showError(err.message || 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const handleChange = (field, value) => {
    // When disabling show_private_contributions, also disable hide_private_repo_names and clear warning
    if (field === 'show_private_contributions' && value === false) {
      setShowRequirementWarning(false);
      setSettings((prev) => ({
        ...prev,
        show_private_contributions: false,
        hide_private_repo_names: false,
      }));
      return;
    }

    // When enabling show_private_contributions, clear warning
    if (field === 'show_private_contributions' && value === true) {
      setShowRequirementWarning(false);
    }

    setSettings((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleDisabledToggleClick = () => {
    // Show warning when user tries to click the disabled toggle
    if (!settings.show_private_contributions) {
      setShowRequirementWarning(true);
    }
  };

  if (loading) {
    return (
      <div className="user-settings">
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="user-settings">
      <div className="settings-page-header">
        <h1>Preferences</h1>
        <p className="settings-page-description">
          Manage your account preferences and privacy settings
        </p>
      </div>

      <form onSubmit={handleSave}>
        {/* Appearance Settings */}
        <div className="settings-section">
          <h2>Appearance</h2>
          <div className="settings-group">
            <div className="setting-item">
              <div className="setting-info">
                <label className="setting-label">Theme</label>
                <p className="setting-description">Choose your preferred color scheme</p>
              </div>
              <div className="setting-control">
                <ThemeToggle />
              </div>
            </div>
          </div>
        </div>

        {/* Privacy Settings */}
        <div className="settings-section">
          <h2>Privacy</h2>
          <div className="settings-group">
            <div className="setting-item">
              <div className="setting-info">
                <label className="setting-label">Show Private Contributions</label>
                <p className="setting-description">
                  Include contributions from private repositories in your heatmap
                </p>
              </div>
              <div className="setting-control">
                <label className="toggle-switch">
                  <input
                    type="checkbox"
                    checked={settings.show_private_contributions}
                    onChange={(e) =>
                      handleChange('show_private_contributions', e.target.checked)
                    }
                  />
                  <span className="toggle-slider"></span>
                </label>
              </div>
            </div>

            <div className="setting-item">
              <div className="setting-info">
                <label className="setting-label">Hide Private Repository Names</label>
                <p className="setting-description">
                  Show contribution counts from private repos but hide repository names
                </p>
                {showRequirementWarning && (
                  <p className="setting-requirement">
                    ⚠️ Please enable "Show Private Contributions" first to use this option
                  </p>
                )}
              </div>
              <div className="setting-control">
                <label
                  className="toggle-switch"
                  onClick={handleDisabledToggleClick}
                >
                  <input
                    type="checkbox"
                    checked={settings.hide_private_repo_names}
                    onChange={(e) => handleChange('hide_private_repo_names', e.target.checked)}
                    disabled={!settings.show_private_contributions}
                  />
                  <span className="toggle-slider"></span>
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Save Button */}
        <div className="settings-actions">
          <button type="submit" className="btn btn-primary" disabled={saving}>
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  );
}
