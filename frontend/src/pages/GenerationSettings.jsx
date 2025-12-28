import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import apiClient from '../api/client';
import './GenerationSettings.css';

function GenerationSettings() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(false);
  const [jobs, setJobs] = useState([]);

  const [settings, setSettings] = useState({
    update_interval_minutes: 60,
    auto_generation_enabled: true,
    date_range_days: 365,
    include_private_contributions: true,
  });

  useEffect(() => {
    loadSettings();
    loadJobs();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const data = await apiClient.getGenerationSettings();
      setSettings({
        update_interval_minutes: data.update_interval_minutes,
        auto_generation_enabled: data.auto_generation_enabled,
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

  const loadJobs = async () => {
    try {
      const data = await apiClient.listGenerationJobs(null, 20);
      setJobs(data);
    } catch (err) {
      console.error('Failed to load jobs:', err);
    }
  };

  const handleChange = (field, value) => {
    setSettings(prev => ({ ...prev, [field]: value }));
    setSuccess(false);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    try {
      setSaving(true);
      setError(null);
      await apiClient.updateGenerationSettings(settings);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  };

  const handleTriggerGeneration = async () => {
    try {
      await apiClient.triggerGeneration();
      alert('Heatmap generation started! Check the jobs list below.');
      await loadJobs();
    } catch (err) {
      alert(`Failed to trigger generation: ${err.message}`);
    }
  };

  const getStatusBadgeClass = (status) => {
    switch (status) {
      case 'completed':
        return 'status-completed';
      case 'processing':
        return 'status-processing';
      case 'failed':
        return 'status-failed';
      default:
        return 'status-pending';
    }
  };

  const formatDate = (dateString) => {
    if (!dateString) return 'N/A';
    return new Date(dateString).toLocaleString();
  };

  if (loading) {
    return (
      <div className="generation-settings-container">
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="generation-settings-container">
      <div className="settings-header">
        <h1>Generation Settings</h1>
        <button className="btn-secondary" onClick={() => navigate('/settings/themes')}>
          Back to Themes
        </button>
      </div>

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">Settings saved successfully!</div>}

      <div className="settings-content">
        <form onSubmit={handleSubmit} className="settings-form">
          <section className="form-section">
            <h2>Automatic Generation</h2>

            <div className="form-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.auto_generation_enabled}
                  onChange={(e) => handleChange('auto_generation_enabled', e.target.checked)}
                />
                <span>Enable automatic heatmap generation</span>
              </label>
              <p className="hint-text">
                When enabled, heatmaps will be automatically regenerated based on the update interval below.
              </p>
            </div>

            {settings.auto_generation_enabled && (
              <div className="form-group">
                <label>Update Interval (minutes)</label>
                <select
                  value={settings.update_interval_minutes}
                  onChange={(e) => handleChange('update_interval_minutes', parseInt(e.target.value))}
                >
                  <option value={15}>15 minutes</option>
                  <option value={30}>30 minutes</option>
                  <option value={60}>1 hour</option>
                  <option value={180}>3 hours</option>
                  <option value={360}>6 hours</option>
                  <option value={720}>12 hours</option>
                  <option value={1440}>24 hours</option>
                </select>
                <p className="hint-text">
                  How often to regenerate heatmaps automatically.
                </p>
              </div>
            )}
          </section>

          <section className="form-section">
            <h2>Data Settings</h2>

            <div className="form-group">
              <label>Date Range (days)</label>
              <input
                type="number"
                min="1"
                max="730"
                value={settings.date_range_days}
                onChange={(e) => handleChange('date_range_days', parseInt(e.target.value))}
              />
              <p className="hint-text">
                How many days of contribution data to include in the heatmap (1-730 days). Common values: 90 (3 months), 180 (6 months), 365 (1 year).
              </p>
            </div>

            <div className="form-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.include_private_contributions}
                  onChange={(e) => handleChange('include_private_contributions', e.target.checked)}
                />
                <span>Include private contributions</span>
              </label>
              <p className="hint-text">
                Whether to include contributions from private repositories.
              </p>
            </div>
          </section>

          <div className="form-actions">
            <button type="submit" className="btn-primary" disabled={saving}>
              {saving ? 'Saving...' : 'Save Settings'}
            </button>
          </div>
        </form>

        {/* Manual Generation */}
        <section className="manual-generation">
          <h2>Manual Generation</h2>
          <p>Trigger immediate regeneration of all heatmaps, regardless of the automatic schedule.</p>
          <button className="btn-primary" onClick={handleTriggerGeneration}>
            🔄 Generate All Heatmaps Now
          </button>
        </section>

        {/* Recent Jobs */}
        <section className="recent-jobs">
          <h2>Recent Generation Jobs</h2>
          {jobs.length === 0 ? (
            <p className="empty-state">No generation jobs yet.</p>
          ) : (
            <div className="jobs-table">
              <table>
                <thead>
                  <tr>
                    <th>Status</th>
                    <th>Theme</th>
                    <th>Scheduled</th>
                    <th>Started</th>
                    <th>Completed</th>
                    <th>Type</th>
                  </tr>
                </thead>
                <tbody>
                  {jobs.map((job) => (
                    <tr key={job.id}>
                      <td>
                        <span className={`status-badge ${getStatusBadgeClass(job.status)}`}>
                          {job.status}
                        </span>
                      </td>
                      <td>{job.theme_id ? 'Single Theme' : 'All Themes'}</td>
                      <td>{formatDate(job.scheduled_at)}</td>
                      <td>{formatDate(job.started_at)}</td>
                      <td>{formatDate(job.completed_at)}</td>
                      <td>
                        <span className={job.is_manual ? 'badge-manual' : 'badge-auto'}>
                          {job.is_manual ? 'Manual' : 'Auto'}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

export default GenerationSettings;
