import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import apiClient from '../api/client';
import ThemeToggle from '../components/ThemeToggle';
import './Settings.css';

function Settings() {
  const { user } = useAuth();
  const [settings, setSettings] = useState({
    show_private_contributions: true,
    hide_private_repo_names: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);
  const [successMessage, setSuccessMessage] = useState(null);

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
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (e) => {
    e.preventDefault();

    try {
      setSaving(true);
      setError(null);
      setSuccessMessage(null);

      await apiClient.updateSettings(settings);

      setSuccessMessage('Settings saved successfully!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleChange = (field, value) => {
    setSettings((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  if (loading) {
    return (
      <div className="settings-page">
        <div className="settings-loading">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="settings-page">
      <div className="settings-header">
        <div className="settings-breadcrumb">
          <Link to="/" className="breadcrumb-link">
            Dashboard
          </Link>
          <span className="breadcrumb-separator">/</span>
          <span className="breadcrumb-current">Settings</span>
        </div>
        <h1>Settings</h1>
      </div>

      <div className="settings-container">
        {error && <div className="error-message">{error}</div>}
        {successMessage && <div className="success-message">{successMessage}</div>}

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
                </div>
                <div className="setting-control">
                  <label className="toggle-switch">
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

          {/* Admin Settings - Only visible to admins */}
          {user?.is_admin && (
            <div className="settings-section admin-section">
              <h2>Administration</h2>
              <div className="settings-group">
                <div className="setting-item">
                  <div className="setting-info">
                    <label className="setting-label">OAuth Applications</label>
                    <p className="setting-description">
                      Manage OAuth applications for GitHub, GitLab, and Gitea integrations. Configure OAuth
                      credentials to allow users to connect their accounts without editing .env files.
                    </p>
                  </div>
                  <div className="setting-control">
                    <Link to="/admin/oauth-apps" className="btn btn-admin">
                      Manage OAuth Apps →
                    </Link>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Save Button */}
          <div className="settings-actions">
            <button type="submit" className="btn btn-primary" disabled={saving}>
              {saving ? 'Saving...' : 'Save Changes'}
            </button>
            <Link to="/" className="btn btn-secondary">
              Cancel
            </Link>
          </div>
        </form>
      </div>
    </div>
  );
}

export default Settings;
