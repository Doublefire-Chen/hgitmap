import { useState, useEffect } from 'react';
import './HeatmapPreview.css';

function HeatmapPreview({ theme }) {
  const [showPreview, setShowPreview] = useState(false);
  const [previewSvg, setPreviewSvg] = useState(null);
  const [previewImageUrl, setPreviewImageUrl] = useState(null);
  const [previewFormat, setPreviewFormat] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [previewPeriod, setPreviewPeriod] = useState('rolling'); // 'rolling' or specific year

  // Track if theme has changed since last preview generation
  const [themeChanged, setThemeChanged] = useState(false);
  const [lastGeneratedTheme, setLastGeneratedTheme] = useState(null);

  // Cleanup object URL when component unmounts
  useEffect(() => {
    return () => {
      if (previewImageUrl) {
        URL.revokeObjectURL(previewImageUrl);
      }
    };
  }, [previewImageUrl]);

  // Detect if theme has changed since last generation
  useEffect(() => {
    if (lastGeneratedTheme && showPreview) {
      const hasChanged = JSON.stringify(theme) !== JSON.stringify(lastGeneratedTheme);
      setThemeChanged(hasChanged);
    }
  }, [theme, lastGeneratedTheme, showPreview]);

  const handleTogglePreview = async () => {
    if (showPreview && !themeChanged) {
      setShowPreview(false);
      // Clean up image URL if exists
      if (previewImageUrl) {
        URL.revokeObjectURL(previewImageUrl);
        setPreviewImageUrl(null);
      }
      return;
    }

    // Generate preview by calling backend
    setLoading(true);
    setError(null);

    try {
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
      const token = localStorage.getItem('token');

      // Calculate date range based on selected period
      let fromDate, toDate;
      if (previewPeriod === 'rolling') {
        // Last year (rolling 365 days)
        const today = new Date();
        const windowStart = new Date(today);
        windowStart.setDate(windowStart.getDate() - 364);

        // Find the Sunday before (or on) the window start
        const dayOfWeek = windowStart.getDay();
        const from = new Date(windowStart);
        if (dayOfWeek !== 0) {
          from.setDate(from.getDate() - dayOfWeek);
        }

        const formatLocalDate = (date) => {
          const year = date.getFullYear();
          const month = String(date.getMonth() + 1).padStart(2, '0');
          const day = String(date.getDate()).padStart(2, '0');
          return `${year}-${month}-${day}`;
        };

        fromDate = formatLocalDate(from);
        toDate = formatLocalDate(today);
      } else {
        // Specific year
        const year = parseInt(previewPeriod);
        fromDate = `${year}-01-01`;
        const today = new Date();
        if (year === today.getFullYear()) {
          // Current year: up to today
          const month = String(today.getMonth() + 1).padStart(2, '0');
          const day = String(today.getDate()).padStart(2, '0');
          toDate = `${year}-${month}-${day}`;
        } else {
          // Past year: full year
          toDate = `${year}-12-31`;
        }
      }

      // Get the output format (single selection)
      const outputFormat = theme.output_format || 'svg';

      const requestBody = {
        ...theme,
        preview_from_date: fromDate,
        preview_to_date: toDate,
        preview_format: outputFormat
      };

      const response = await fetch(`${API_BASE_URL}/heatmap/preview`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(requestBody)
      });

      if (!response.ok) {
        throw new Error('Failed to generate preview');
      }

      // Handle different formats
      if (outputFormat === 'svg') {
        const svgText = await response.text();
        setPreviewSvg(svgText);
        setPreviewFormat('svg');
      } else {
        // For PNG, JPEG, WebP - create object URL from blob
        const blob = await response.blob();
        const objectUrl = URL.createObjectURL(blob);
        setPreviewImageUrl(objectUrl);
        setPreviewFormat(outputFormat);
      }

      setShowPreview(true);
      setLastGeneratedTheme(theme); // Save theme to detect future changes
      setThemeChanged(false); // Reset the changed flag
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  // Generate year options from 2020 to current year
  const currentYear = new Date().getFullYear();
  const years = Array.from({ length: currentYear - 2019 }, (_, i) => currentYear - i);

  return (
    <div className="heatmap-preview">
      <div className="heatmap-preview-inner">
        <div className="preview-header">
          <h3>Preview</h3>
          <div className="preview-controls">
            <div className="preview-period-selector">
              <label>Period:</label>
              <select
                value={previewPeriod}
                onChange={(e) => setPreviewPeriod(e.target.value)}
                className="period-select"
              >
                <option value="rolling">Last year</option>
                {years.map(year => (
                  <option key={year} value={year}>{year}</option>
                ))}
              </select>
            </div>
            <button
              type="button"
              className="btn-preview"
              onClick={handleTogglePreview}
              disabled={loading}
            >
              {loading ? '⏳ Generating...' :
                showPreview && themeChanged ? '🔄 Update Preview' :
                  showPreview ? '🔼 Hide Preview' :
                    '🔽 Show Preview'}
            </button>
          </div>
        </div>

        {error && (
          <div className="preview-error">
            ⚠️ Failed to generate preview: {error}
          </div>
        )}

        {showPreview && (previewSvg || previewImageUrl) && (
          <div className="preview-content">
            {themeChanged && (
              <p className="preview-hint" style={{ background: '#fff3cd', borderColor: '#ffc107', color: '#856404' }}>
                ⚠️ Theme settings have changed. Click "Update Preview" to see the latest changes.
              </p>
            )}
            <p className="preview-hint">
              This preview uses your actual contribution data. The generated heatmap will look exactly like this.
            </p>
            <div className="preview-svg-container">
              {previewFormat === 'svg' ? (
                <div dangerouslySetInnerHTML={{ __html: previewSvg }} />
              ) : (
                <img src={previewImageUrl} alt="Heatmap Preview" />
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default HeatmapPreview;
