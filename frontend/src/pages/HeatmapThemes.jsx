import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import apiClient from '../api/client';
import './HeatmapThemes.css';

function HeatmapThemes() {
  const navigate = useNavigate();
  const [themes, setThemes] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [generatedHeatmaps, setGeneratedHeatmaps] = useState([]);
  const user = JSON.parse(localStorage.getItem('user') || '{}');

  useEffect(() => {
    loadThemes();
    loadGeneratedHeatmaps();
  }, []);

  const loadThemes = async () => {
    try {
      setLoading(true);
      const data = await apiClient.listThemes();
      setThemes(data);
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const loadGeneratedHeatmaps = async () => {
    try {
      const data = await apiClient.listGeneratedHeatmaps();
      setGeneratedHeatmaps(data);
    } catch (err) {
      console.error('Failed to load generated heatmaps:', err);
    }
  };

  const handleDelete = async (slug, themeName) => {
    if (!window.confirm(`Are you sure you want to delete theme "${themeName}"?`)) {
      return;
    }

    try {
      await apiClient.deleteTheme(slug);
      await loadThemes();
    } catch (err) {
      alert(`Failed to delete theme: ${err.message}`);
    }
  };

  const handleSetDefault = async (slug) => {
    try {
      await apiClient.setDefaultTheme(slug);
      await loadThemes();
    } catch (err) {
      alert(`Failed to set default theme: ${err.message}`);
    }
  };

  const handleGenerate = async (slug) => {
    try {
      await apiClient.triggerThemeGeneration(slug);
      alert('Heatmap generation started! Check the generation jobs for status.');
    } catch (err) {
      alert(`Failed to trigger generation: ${err.message}`);
    }
  };

  const handleDuplicate = async (slug, themeName) => {
    const newName = prompt(`Enter a name for the duplicated theme:`, `${themeName} (Copy)`);
    if (!newName) return;

    // Generate slug from name
    const newSlug = newName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

    if (!newSlug) {
      alert('Invalid theme name');
      return;
    }

    try {
      await apiClient.duplicateTheme(slug, newName, newSlug);
      await loadThemes();
      alert(`Theme "${newName}" duplicated successfully!`);
    } catch (err) {
      alert(`Failed to duplicate theme: ${err.message}`);
    }
  };

  const getGeneratedHeatmap = (themeId, format) => {
    return generatedHeatmaps.find(
      h => h.theme_id === themeId && h.format === format && h.is_valid
    );
  };

  const getEmbedUrl = (slug, format) => {
    return apiClient.getHeatmapEmbedUrl(user.username, slug, format);
  };

  if (loading) {
    return (
      <div className="heatmap-themes-container">
        <div className="loading">Loading themes...</div>
      </div>
    );
  }

  return (
    <div className="heatmap-themes-container">
      <div className="themes-header">
        <h1>Heatmap Themes</h1>
        <div className="header-actions">
          <button
            className="btn-secondary"
            onClick={() => navigate('/heatmap/settings')}
          >
            Generation Settings
          </button>
          <button
            className="btn-primary"
            onClick={() => navigate('/heatmap/themes/new')}
          >
            + New Theme
          </button>
        </div>
      </div>

      {error && <div className="error-message">{error}</div>}

      {themes.length === 0 ? (
        <div className="empty-state">
          <p>No themes found. Create your first heatmap theme!</p>
          <button
            className="btn-primary"
            onClick={() => navigate('/heatmap/themes/new')}
          >
            Create Theme
          </button>
        </div>
      ) : (
        <div className="themes-grid">
          {themes.map((theme) => (
            <div key={theme.id} className="theme-card">
              <div className="theme-card-header">
                <h3>
                  {theme.name}
                  {theme.is_default && (
                    <span className="badge-default">Default</span>
                  )}
                </h3>
                <span className={`theme-mode-badge ${theme.theme_mode}`}>
                  {theme.theme_mode}
                </span>
              </div>

              {theme.description && (
                <p className="theme-description">{theme.description}</p>
              )}

              <div className="theme-details">
                <div className="detail-row">
                  <span className="label">Color Scheme:</span>
                  <span className="value">{theme.color_scheme.replace('_', ' ')}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Cell Size:</span>
                  <span className="value">{theme.cell_size}px</span>
                </div>
                <div className="detail-row">
                  <span className="label">Formats:</span>
                  <span className="value">{theme.output_formats.join(', ').toUpperCase()}</span>
                </div>
                <div className="detail-row">
                  <span className="label">Slug:</span>
                  <span className="value monospace">{theme.slug}</span>
                </div>
              </div>

              {/* Preview Image if generated */}
              {generatedHeatmaps.find(
                h => h.theme_id === theme.id && h.is_valid
              ) && (
                <div className="theme-preview">
                  <img
                    src={getEmbedUrl(theme.slug, theme.output_formats[0])}
                    alt={`${theme.name} preview`}
                    onError={(e) => {
                      e.target.style.display = 'none';
                    }}
                  />
                </div>
              )}

              <div className="theme-actions">
                <button
                  className="btn-icon"
                  onClick={() => navigate(`/heatmap/themes/${theme.slug}/edit`)}
                  title="Edit theme"
                >
                  ✏️ Edit
                </button>
                <button
                  className="btn-icon"
                  onClick={() => handleDuplicate(theme.slug, theme.name)}
                  title="Duplicate theme"
                >
                  📋 Duplicate
                </button>
                <button
                  className="btn-icon"
                  onClick={() => handleGenerate(theme.slug)}
                  title="Generate heatmap"
                >
                  🔄 Generate
                </button>
                {!theme.is_default && (
                  <button
                    className="btn-icon"
                    onClick={() => handleSetDefault(theme.slug)}
                    title="Set as default"
                  >
                    ⭐ Set Default
                  </button>
                )}
                {!theme.is_default && (
                  <button
                    className="btn-icon btn-danger"
                    onClick={() => handleDelete(theme.slug, theme.name)}
                    title="Delete theme"
                  >
                    🗑️ Delete
                  </button>
                )}
              </div>

              {/* Embed URLs */}
              <div className="embed-urls">
                <h4>Embed URLs:</h4>
                {theme.output_formats.map(format => (
                  <div key={format} className="embed-url-row">
                    <code>{getEmbedUrl(theme.slug, format)}</code>
                    <button
                      className="btn-copy"
                      onClick={() => {
                        navigator.clipboard.writeText(getEmbedUrl(theme.slug, format));
                        alert('Copied to clipboard!');
                      }}
                    >
                      📋
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default HeatmapThemes;
