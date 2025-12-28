import { useState, useEffect } from 'react';
import { useToast } from '../context/ToastContext';
import apiClient from '../api/client';
import PlatformIcon from './PlatformIcon';
import { FiEdit, FiTrash2, FiArrowLeft } from 'react-icons/fi';
import './OAuthAppsManager.css';

export default function OAuthAppsManager() {
  const { error: showError, success: showSuccess } = useToast();
  const [apps, setApps] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editingApp, setEditingApp] = useState(null);

  // Form state
  const [formData, setFormData] = useState({
    platform: 'github',
    instanceName: '',
    instanceUrl: '',
    clientId: '',
    clientSecret: '',
    isDefault: true,
  });

  useEffect(() => {
    loadApps();
  }, []);

  const loadApps = async () => {
    try {
      setLoading(true);
      const data = await apiClient.listOAuthApps();
      setApps(data);
    } catch (err) {
      showError(err.message || 'Failed to load OAuth apps');
    } finally {
      setLoading(false);
    }
  };

  const handleInputChange = (e) => {
    const { name, value, type, checked } = e.target;
    setFormData({
      ...formData,
      [name]: type === 'checkbox' ? checked : value,
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    try {
      if (editingApp) {
        // Update existing app
        const updates = {};
        if (formData.instanceName) updates.instance_name = formData.instanceName;
        if (formData.clientId) updates.client_id = formData.clientId;
        if (formData.clientSecret) updates.client_secret = formData.clientSecret;
        if (formData.isDefault !== undefined) updates.is_default = formData.isDefault;

        await apiClient.updateOAuthApp(editingApp.id, updates);
        showSuccess('OAuth app updated successfully!');
      } else {
        // Create new app
        await apiClient.createOAuthApp(
          formData.platform,
          formData.instanceName,
          formData.instanceUrl,
          formData.clientId,
          formData.clientSecret,
          formData.isDefault
        );
        showSuccess('OAuth app created successfully!');
      }

      // Reset form and reload apps
      cancelForm();
      loadApps();
    } catch (err) {
      showError(err.message || 'Failed to save OAuth app');
    }
  };

  const handleEdit = (app) => {
    setEditingApp(app);
    setFormData({
      platform: app.platform,
      instanceName: app.instance_name,
      instanceUrl: app.instance_url,
      clientId: app.client_id,
      clientSecret: '', // Don't pre-fill secret
      isDefault: app.is_default,
    });
    setShowForm(true);
  };

  const handleDelete = async (appId) => {
    if (!confirm('Are you sure you want to delete this OAuth app? Users will no longer be able to use it.')) {
      return;
    }

    try {
      await apiClient.deleteOAuthApp(appId);
      showSuccess('OAuth app deleted successfully!');
      loadApps();
    } catch (err) {
      showError(err.message || 'Failed to delete OAuth app');
    }
  };

  const handleToggleEnabled = async (app) => {
    try {
      await apiClient.updateOAuthApp(app.id, {
        is_enabled: !app.is_enabled,
      });
      showSuccess(`OAuth app ${app.is_enabled ? 'disabled' : 'enabled'} successfully!`);
      loadApps();
    } catch (err) {
      showError(err.message || 'Failed to update OAuth app');
    }
  };

  const cancelForm = () => {
    setShowForm(false);
    setEditingApp(null);
    setFormData({
      platform: 'github',
      instanceName: '',
      instanceUrl: '',
      clientId: '',
      clientSecret: '',
      isDefault: true,
    });
  };

  if (loading) {
    return (
      <div className="oauth-apps-manager">
        <div className="loading">Loading OAuth applications...</div>
      </div>
    );
  }

  return (
    <div className="oauth-apps-manager">
      <div className="oauth-apps-content">
        {!showForm ? (
          <div className="apps-list-section">
            <div className="section-header">
              <h2>Configured OAuth Applications</h2>
              <button
                onClick={() => setShowForm(true)}
                className="btn btn-primary"
              >
                + Add OAuth App
              </button>
            </div>

          {apps.length === 0 ? (
            <div className="empty-state">
              <p>No OAuth applications configured.</p>
              <p className="hint">
                Add an OAuth app to allow users to connect their Git platform accounts without using Personal Access Tokens.
              </p>
            </div>
          ) : (
            <div className="apps-list">
              {apps.map((app) => (
                <div key={app.id} className={`app-card ${!app.is_enabled ? 'disabled' : ''}`}>
                  <div className="app-header">
                    <div className="app-title">
                      <div className="app-title-row">
                        <PlatformIcon platform={app.platform} size={24} />
                        <h3>{app.instance_name}</h3>
                      </div>
                      <div className="app-badges">
                        {app.is_default && <span className="badge badge-default">Default</span>}
                        {!app.is_enabled && <span className="badge badge-disabled">Disabled</span>}
                        <label className="toggle-switch" title={app.is_enabled ? 'Enabled - Click to disable' : 'Disabled - Click to enable'}>
                          <input
                            type="checkbox"
                            checked={app.is_enabled}
                            onChange={() => handleToggleEnabled(app)}
                            aria-label={app.is_enabled ? 'Disable OAuth app' : 'Enable OAuth app'}
                          />
                          <span className="toggle-slider"></span>
                        </label>
                      </div>
                    </div>
                    <div className="url-actions-row">
                      <div className="instance-url-display">
                        <span className="instance-url-text">
                          {app.instance_url || (app.platform === 'github' ? 'https://github.com' : `https://${app.platform}.com`)}
                        </span>
                      </div>
                      <div className="app-actions">
                        <div
                          onClick={() => handleEdit(app)}
                          className="btn-icon btn-icon-secondary"
                          title="Edit"
                          role="button"
                          tabIndex={0}
                          onKeyDown={(e) => e.key === 'Enter' && handleEdit(app)}
                          aria-label="Edit OAuth app"
                        >
                          <FiEdit size={18} />
                        </div>

                        <div
                          onClick={() => handleDelete(app.id)}
                          className="btn-icon btn-icon-danger"
                          title="Delete"
                          role="button"
                          tabIndex={0}
                          onKeyDown={(e) => e.key === 'Enter' && handleDelete(app.id)}
                          aria-label="Delete OAuth app"
                        >
                          <FiTrash2 size={18} />
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
        ) : (
          <div className="form-section">
            <div className="form-header">
              <button onClick={cancelForm} className="btn-back">
                <FiArrowLeft size={20} />
                <span>Back to OAuth Apps</span>
              </button>
              <h2>{editingApp ? 'Edit OAuth App' : 'Add OAuth App'}</h2>
            </div>
            <form onSubmit={handleSubmit}>
              <div className="form-group">
                <label htmlFor="platform">Platform</label>
                <select
                  id="platform"
                  name="platform"
                  value={formData.platform}
                  onChange={handleInputChange}
                  disabled={editingApp !== null}
                  required
                >
                  <option value="github">GitHub</option>
                  <option value="gitlab">GitLab</option>
                  <option value="gitea">Gitea</option>
                </select>
                {editingApp && (
                  <small className="form-hint">Platform cannot be changed after creation</small>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="instanceName">Instance Name</label>
                <input
                  type="text"
                  id="instanceName"
                  name="instanceName"
                  value={formData.instanceName}
                  onChange={handleInputChange}
                  placeholder="e.g., GitHub, Company GitLab"
                  required
                />
                <small className="form-hint">A friendly name for this OAuth app</small>
              </div>

              {formData.platform !== 'github' && (
                <div className="form-group">
                  <label htmlFor="instanceUrl">Instance URL</label>
                  <input
                    type="url"
                    id="instanceUrl"
                    name="instanceUrl"
                    value={formData.instanceUrl}
                    onChange={handleInputChange}
                    placeholder="https://gitlab.example.com"
                    disabled={editingApp !== null}
                  />
                  <small className="form-hint">
                    Leave empty for official {formData.platform} (gitlab.com, etc.)
                  </small>
                  {editingApp && (
                    <small className="form-hint">Instance URL cannot be changed after creation</small>
                  )}
                </div>
              )}

              <div className="form-group">
                <label htmlFor="clientId">Client ID</label>
                <input
                  type="text"
                  id="clientId"
                  name="clientId"
                  value={formData.clientId}
                  onChange={handleInputChange}
                  placeholder="Iv1.abc123..."
                  required={!editingApp}
                />
                <small className="form-hint">
                  {editingApp
                    ? 'Leave empty to keep the existing Client ID'
                    : 'OAuth application Client ID from your Git platform'}
                </small>
              </div>

              <div className="form-group">
                <label htmlFor="clientSecret">Client Secret</label>
                <input
                  type="password"
                  id="clientSecret"
                  name="clientSecret"
                  value={formData.clientSecret}
                  onChange={handleInputChange}
                  placeholder="ghp_xyz789..."
                  required={!editingApp}
                  autoComplete="new-password"
                />
                <small className="form-hint">
                  {editingApp
                    ? 'Leave empty to keep the existing Client Secret'
                    : 'OAuth application Client Secret (will be encrypted)'}
                </small>
              </div>

              <div className="form-group checkbox-group">
                <label>
                  <input
                    type="checkbox"
                    name="isDefault"
                    checked={formData.isDefault}
                    onChange={handleInputChange}
                  />
                  Set as default OAuth app for this platform
                </label>
                <small className="form-hint">
                  The default app will be used when users click "Connect with OAuth"
                </small>
              </div>

              <div className="form-actions">
                <button type="submit" className="btn btn-primary">
                  {editingApp ? 'Update OAuth App' : 'Create OAuth App'}
                </button>
                <button type="button" onClick={cancelForm} className="btn btn-secondary">
                  Cancel
                </button>
              </div>
            </form>
          </div>
        )}
      </div>
    </div>
  );
}
