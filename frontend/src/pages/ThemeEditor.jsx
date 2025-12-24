import { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import apiClient from '../api/client';
import ColorSchemePicker from '../components/ColorSchemePicker';
import HeatmapPreview from '../components/HeatmapPreview';
import './ThemeEditor.css';

const FORMATS = ['svg', 'png', 'jpeg', 'webp'];

function ThemeEditor() {
  const navigate = useNavigate();
  const { slug } = useParams();
  const isEditing = !!slug;

  const [loading, setLoading] = useState(isEditing);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);

  const [formData, setFormData] = useState({
    name: '',
    slug: '',
    description: '',
    theme_mode: 'light',
    color_scheme: 'github_green',
    custom_colors: ['#ebedf0', '#9be9a8', '#40c463', '#30a14e', '#216e39'],

    // Colors
    background_color: '#ffffff',
    border_color: '#d1d5da',
    text_color: '#24292e',
    empty_cell_color: '#ebedf0',
    cell_border_color: '#d1d5da',

    // Cell styling
    cell_size: 10,
    cell_gap: 2,
    cell_border_radius: 2,
    cell_border_width: 0,

    // Dimensions
    heatmap_width: null,
    heatmap_height: null,
    padding_top: 20,
    padding_right: 20,
    padding_bottom: 20,
    padding_left: 20,

    // Display options
    show_month_labels: true,
    show_day_labels: true,
    show_legend: true,
    show_total_count: true,

    // Font
    font_family: 'sans-serif',
    font_size: 10,
    legend_position: 'bottom',

    // Format (single selection)
    output_format: 'png',
  });

  useEffect(() => {
    if (isEditing) {
      loadTheme();
    }
  }, [slug]);

  const loadTheme = async () => {
    try {
      setLoading(true);
      const theme = await apiClient.getTheme(slug);

      // Convert backend format to form format
      setFormData({
        name: theme.name,
        slug: theme.slug,
        description: theme.description || '',
        theme_mode: theme.theme_mode,
        color_scheme: theme.color_scheme,
        custom_colors: theme.custom_colors || formData.custom_colors,
        background_color: theme.background_color,
        border_color: theme.border_color,
        text_color: theme.text_color,
        empty_cell_color: theme.empty_cell_color,
        cell_border_color: theme.cell_border_color,
        cell_size: theme.cell_size,
        cell_gap: theme.cell_gap,
        cell_border_radius: theme.cell_border_radius,
        cell_border_width: theme.cell_border_width,
        heatmap_width: theme.heatmap_width,
        heatmap_height: theme.heatmap_height,
        padding_top: theme.padding_top,
        padding_right: theme.padding_right,
        padding_bottom: theme.padding_bottom,
        padding_left: theme.padding_left,
        show_month_labels: theme.show_month_labels,
        show_day_labels: theme.show_day_labels,
        show_legend: theme.show_legend,
        show_total_count: theme.show_total_count,
        font_family: theme.font_family,
        font_size: theme.font_size,
        legend_position: theme.legend_position,
        output_format: theme.output_formats && theme.output_formats.length > 0
          ? theme.output_formats[0]
          : 'png',
      });

      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (field, value) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleNumberChange = (field, value) => {
    const num = value === '' ? null : parseInt(value, 10);
    setFormData(prev => ({ ...prev, [field]: num }));
  };

  const handleCustomColorChange = (index, value) => {
    const newColors = [...formData.custom_colors];
    newColors[index] = value;
    setFormData(prev => ({ ...prev, custom_colors: newColors }));
  };

  const handleThemeModeChange = (mode) => {
    const defaults = mode === 'dark'
      ? {
          background_color: '#0d1117',
          text_color: '#c9d1d9',
          empty_cell_color: '#161b22',
          border_color: '#30363d',
        }
      : {
          background_color: '#ffffff',
          text_color: '#24292e',
          empty_cell_color: '#ebedf0',
          border_color: '#d1d5da',
        };

    setFormData(prev => ({ ...prev, theme_mode: mode, ...defaults }));
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    try {
      setSaving(true);
      setError(null);

      // Convert single format to array for backend
      const submitData = {
        ...formData,
        output_formats: [formData.output_format]
      };
      delete submitData.output_format;

      if (isEditing) {
        await apiClient.updateTheme(slug, submitData);
      } else {
        await apiClient.createTheme(submitData);
      }

      navigate('/heatmap/themes');
    } catch (err) {
      setError(err.message);
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="theme-editor-container"><div className="loading">Loading theme...</div></div>;
  }

  return (
    <div className="theme-editor-container">
      <div className="editor-header">
        <h1>{isEditing ? 'Edit Theme' : 'Create New Theme'}</h1>
        <button className="btn-secondary" onClick={() => navigate('/heatmap/themes')}>
          Cancel
        </button>
      </div>

      {error && <div className="error-message">{error}</div>}

      <form onSubmit={handleSubmit} className="theme-form">
        {/* Basic Information */}
        <section className="form-section">
          <h2>Basic Information</h2>

          <div className="form-group">
            <label>Theme Name *</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => handleChange('name', e.target.value)}
              required
              placeholder="My Awesome Theme"
            />
          </div>

          <div className="form-group">
            <label>Slug * <span className="hint">(URL-safe identifier)</span></label>
            <input
              type="text"
              value={formData.slug}
              onChange={(e) => handleChange('slug', e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-'))}
              required
              disabled={isEditing}
              placeholder="my-awesome-theme"
            />
            {isEditing && <p className="hint-text">Slug cannot be changed after creation</p>}
          </div>

          <div className="form-group">
            <label>Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => handleChange('description', e.target.value)}
              placeholder="Optional description of your theme"
              rows={3}
            />
          </div>

          <div className="form-group">
            <label>Theme Mode</label>
            <div className="radio-group">
              <label className="radio-option">
                <input
                  type="radio"
                  checked={formData.theme_mode === 'light'}
                  onChange={() => handleThemeModeChange('light')}
                />
                <span>Light</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  checked={formData.theme_mode === 'dark'}
                  onChange={() => handleThemeModeChange('dark')}
                />
                <span>Dark</span>
              </label>
            </div>
          </div>
        </section>

        {/* Color Configuration */}
        <section className="form-section">
          <h2>Colors</h2>

          <div className="form-group">
            <label>Color Scheme</label>
            <ColorSchemePicker
              value={formData.color_scheme}
              onChange={(scheme) => handleChange('color_scheme', scheme)}
              customColors={formData.custom_colors}
            />
          </div>

          {formData.color_scheme === 'custom' && (
            <div className="form-group">
              <label>Custom Colors (5 levels from low to high)</label>
              <div className="custom-colors-grid">
                {formData.custom_colors.map((color, i) => (
                  <div key={i} className="color-input-group">
                    <label>Level {i}</label>
                    <input
                      type="color"
                      value={color}
                      onChange={(e) => handleCustomColorChange(i, e.target.value)}
                    />
                    <input
                      type="text"
                      value={color}
                      onChange={(e) => handleCustomColorChange(i, e.target.value)}
                      pattern="^#[0-9A-Fa-f]{6}$"
                    />
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="form-row">
            <div className="form-group">
              <label>Background Color</label>
              <div className="color-input">
                <input
                  type="color"
                  value={formData.background_color}
                  onChange={(e) => handleChange('background_color', e.target.value)}
                />
                <input
                  type="text"
                  value={formData.background_color}
                  onChange={(e) => handleChange('background_color', e.target.value)}
                />
              </div>
            </div>

            <div className="form-group">
              <label>Text Color</label>
              <div className="color-input">
                <input
                  type="color"
                  value={formData.text_color}
                  onChange={(e) => handleChange('text_color', e.target.value)}
                />
                <input
                  type="text"
                  value={formData.text_color}
                  onChange={(e) => handleChange('text_color', e.target.value)}
                />
              </div>
            </div>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Empty Cell Color</label>
              <div className="color-input">
                <input
                  type="color"
                  value={formData.empty_cell_color}
                  onChange={(e) => handleChange('empty_cell_color', e.target.value)}
                />
                <input
                  type="text"
                  value={formData.empty_cell_color}
                  onChange={(e) => handleChange('empty_cell_color', e.target.value)}
                />
              </div>
            </div>

            <div className="form-group">
              <label>Border Color</label>
              <div className="color-input">
                <input
                  type="color"
                  value={formData.border_color}
                  onChange={(e) => handleChange('border_color', e.target.value)}
                />
                <input
                  type="text"
                  value={formData.border_color}
                  onChange={(e) => handleChange('border_color', e.target.value)}
                />
              </div>
            </div>
          </div>
        </section>

        {/* Cell Styling */}
        <section className="form-section">
          <h2>Cell Styling</h2>

          <div className="form-row">
            <div className="form-group">
              <label>Cell Size (px)</label>
              <input
                type="number"
                value={formData.cell_size}
                onChange={(e) => handleNumberChange('cell_size', e.target.value)}
                min="5"
                max="50"
              />
            </div>

            <div className="form-group">
              <label>Cell Gap (px)</label>
              <input
                type="number"
                value={formData.cell_gap}
                onChange={(e) => handleNumberChange('cell_gap', e.target.value)}
                min="0"
                max="20"
              />
            </div>

            <div className="form-group">
              <label>Border Radius (px)</label>
              <input
                type="number"
                value={formData.cell_border_radius}
                onChange={(e) => handleNumberChange('cell_border_radius', e.target.value)}
                min="0"
                max="10"
              />
            </div>

            <div className="form-group">
              <label>Border Width (px)</label>
              <input
                type="number"
                value={formData.cell_border_width}
                onChange={(e) => handleNumberChange('cell_border_width', e.target.value)}
                min="0"
                max="5"
              />
            </div>
          </div>

          {formData.cell_border_width > 0 && (
            <div className="form-group">
              <label>Cell Border Color</label>
              <div className="color-input">
                <input
                  type="color"
                  value={formData.cell_border_color}
                  onChange={(e) => handleChange('cell_border_color', e.target.value)}
                />
                <input
                  type="text"
                  value={formData.cell_border_color}
                  onChange={(e) => handleChange('cell_border_color', e.target.value)}
                />
              </div>
            </div>
          )}
        </section>

        {/* Dimensions */}
        <section className="form-section">
          <h2>Heatmap Dimensions</h2>
          <p className="hint-text">
            Optional: Specify custom width and/or height for the generated heatmap.
            Leave empty to use auto-calculated dimensions based on cell size and data.
          </p>

          <div className="form-row">
            <div className="form-group">
              <label>Width (px)</label>
              <input
                type="number"
                value={formData.heatmap_width || ''}
                onChange={(e) => handleNumberChange('heatmap_width', e.target.value)}
                min="100"
                placeholder="Auto"
              />
              <p className="hint-text">Custom width (e.g., 1200)</p>
            </div>

            <div className="form-group">
              <label>Height (px)</label>
              <input
                type="number"
                value={formData.heatmap_height || ''}
                onChange={(e) => handleNumberChange('heatmap_height', e.target.value)}
                min="50"
                placeholder="Auto"
              />
              <p className="hint-text">Custom height (e.g., 300)</p>
            </div>
          </div>
        </section>

        {/* Padding */}
        <section className="form-section">
          <h2>Padding</h2>

          <div className="form-row">
            <div className="form-group">
              <label>Top (px)</label>
              <input
                type="number"
                value={formData.padding_top}
                onChange={(e) => handleNumberChange('padding_top', e.target.value)}
                min="0"
              />
            </div>

            <div className="form-group">
              <label>Right (px)</label>
              <input
                type="number"
                value={formData.padding_right}
                onChange={(e) => handleNumberChange('padding_right', e.target.value)}
                min="0"
              />
            </div>

            <div className="form-group">
              <label>Bottom (px)</label>
              <input
                type="number"
                value={formData.padding_bottom}
                onChange={(e) => handleNumberChange('padding_bottom', e.target.value)}
                min="0"
              />
            </div>

            <div className="form-group">
              <label>Left (px)</label>
              <input
                type="number"
                value={formData.padding_left}
                onChange={(e) => handleNumberChange('padding_left', e.target.value)}
                min="0"
              />
            </div>
          </div>
        </section>

        {/* Display Options */}
        <section className="form-section">
          <h2>Display Options</h2>

          <div className="checkbox-group">
            <label className="checkbox-option">
              <input
                type="checkbox"
                checked={formData.show_month_labels}
                onChange={(e) => handleChange('show_month_labels', e.target.checked)}
              />
              <span>Show Month Labels</span>
            </label>

            <label className="checkbox-option">
              <input
                type="checkbox"
                checked={formData.show_day_labels}
                onChange={(e) => handleChange('show_day_labels', e.target.checked)}
              />
              <span>Show Day Labels</span>
            </label>

            <label className="checkbox-option">
              <input
                type="checkbox"
                checked={formData.show_legend}
                onChange={(e) => handleChange('show_legend', e.target.checked)}
              />
              <span>Show Legend</span>
            </label>

            <label className="checkbox-option">
              <input
                type="checkbox"
                checked={formData.show_total_count}
                onChange={(e) => handleChange('show_total_count', e.target.checked)}
              />
              <span>Show Total Count</span>
            </label>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Font Family</label>
              <input
                type="text"
                value={formData.font_family}
                onChange={(e) => handleChange('font_family', e.target.value)}
                placeholder="sans-serif"
              />
            </div>

            <div className="form-group">
              <label>Font Size (px)</label>
              <input
                type="number"
                value={formData.font_size}
                onChange={(e) => handleNumberChange('font_size', e.target.value)}
                min="8"
                max="24"
              />
            </div>
          </div>
        </section>

        {/* Output Format */}
        <section className="form-section">
          <h2>Output Format *</h2>

          <div className="radio-group">
            {FORMATS.map(format => (
              <label key={format} className="radio-option">
                <input
                  type="radio"
                  name="output_format"
                  checked={formData.output_format === format}
                  onChange={() => handleChange('output_format', format)}
                />
                <span>{format.toUpperCase()}</span>
              </label>
            ))}
          </div>
          <p className="hint-text">Select the format for generated heatmaps</p>
        </section>

        {/* Preview */}
        <HeatmapPreview theme={formData} />

        {/* Submit */}
        <div className="form-actions">
          <button
            type="button"
            className="btn-secondary"
            onClick={() => navigate('/heatmap/themes')}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="btn-primary"
            disabled={saving}
          >
            {saving ? 'Saving...' : isEditing ? 'Update Theme' : 'Create Theme'}
          </button>
        </div>
      </form>
    </div>
  );
}

export default ThemeEditor;
