import { useState, useEffect } from 'react';
import { useToast } from '../context/ToastContext';
import apiClient from '../api/client';
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
        <div className="apps-list-section">
          <div className="section-header">
            <h2>Configured OAuth Applications</h2>
            {!showForm && (
              <button onClick={() => setShowForm(true)} className="btn btn-primary">
                + Add OAuth App
              </button>
            )}
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
                      <h3>{app.instance_name}</h3>
                      <div className="app-badges">
                        <span className={`badge badge-${app.platform.toLowerCase()}`}>
                          {app.platform}
                        </span>
                        {app.is_default && <span className="badge badge-default">Default</span>}
                        {!app.is_enabled && <span className="badge badge-disabled">Disabled</span>}
                      </div>
                    </div>
                  </div>

                  <div className="app-details">
                    {app.instance_url && (
                      <div className="detail-row">
                        <span className="detail-label">Instance URL:</span>
                        <span className="detail-value">{app.instance_url}</span>
                      </div>
                    )}
                    <div className="detail-row">
                      <span className="detail-label">Client ID:</span>
                      <span className="detail-value code">{app.client_id}</span>
                    </div>
                    <div className="detail-row">
                      <span className="detail-label">Client Secret:</span>
                      <span className="detail-value code">{app.client_secret_masked}</span>
                    </div>
                    <div className="detail-row">
                      <span className="detail-label">Created:</span>
                      <span className="detail-value">
                        {new Date(app.created_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>

                  <div className="app-actions">
                    <button
                      onClick={() => handleToggleEnabled(app)}
                      className={`btn btn-sm ${app.is_enabled ? 'btn-warning' : 'btn-success'}`}
                    >
                      {app.is_enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button onClick={() => handleEdit(app)} className="btn btn-sm btn-secondary">
                      Edit
                    </button>
                    <button onClick={() => handleDelete(app.id)} className="btn btn-sm btn-danger">
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {showForm && (
          <div className="form-section">
            <h2>{editingApp ? 'Edit OAuth App' : 'Add OAuth App'}</h2>
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
