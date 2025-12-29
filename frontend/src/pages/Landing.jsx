import { Link } from 'react-router-dom';
import { useMemo } from 'react';
import HeatmapLogo from '../components/HeatmapLogo';
import ThemeToggle from '../components/ThemeToggle';
import './Landing.css';

const Landing = () => {
  // Generate heatmap data with month labels
  const heatmapData = useMemo(() => {
    const weeks = [];
    const monthLabels = [];
    let lastMonth = -1;

    for (let weekIndex = 0; weekIndex < 40; weekIndex++) {
      const week = [];
      for (let dayIndex = 0; dayIndex < 7; dayIndex++) {
        const level = Math.floor(Math.random() * 5);
        // Approximate month for demo (cycles through months)
        const month = Math.floor((weekIndex / 4.33)) % 12;
        week.push({ level, month });
      }
      weeks.push(week);

      // Track month changes for labels
      const weekMonth = week[0].month;
      if (weekMonth !== lastMonth) {
        monthLabels.push({ month: weekMonth, weekIndex });
        lastMonth = weekMonth;
      }
    }

    return { weeks, monthLabels };
  }, []);

  const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  return (
    <div className="landing-container">
      <header className="landing-header">
        <div className="header-content">
          <div className="logo-section">
            <HeatmapLogo size={32} />
            <h1>Hgitmap</h1>
          </div>
          <div className="header-actions">
            <ThemeToggle />
            <Link to="/login" className="btn-secondary">
              Login
            </Link>
            <Link to="/register" className="btn-primary">
              Get Started
            </Link>
          </div>
        </div>
      </header>

      <main className="landing-main">
        <section className="hero-section">
          <div className="hero-content">
            <h2 className="hero-title">
              Unified Contribution Heatmap
              <br />
              <span className="hero-subtitle">Across All Your Git Platforms</span>
            </h2>
            <p className="hero-description">
              Aggregate and visualize your contributions from GitHub, GitLab, and Gitea
              into a single, beautiful heatmap. Track your coding activity across all platforms
              in one place.
            </p>
            <div className="hero-cta">
              <Link to="/register" className="btn-primary btn-large">
                Create Your Profile
              </Link>
              <Link to="/login" className="btn-secondary btn-large">
                Sign In
              </Link>
            </div>
          </div>
          <div className="hero-visual">
            <div className="heatmap-preview">
              {/* Month labels */}
              <div className="heatmap-months">
                {heatmapData.monthLabels.map((label, index) => (
                  <div
                    key={index}
                    className="month-label"
                    style={{ gridColumn: label.weekIndex + 1 }}
                  >
                    {monthNames[label.month]}
                  </div>
                ))}
              </div>

              {/* Grid wrapper with day labels */}
              <div className="heatmap-grid-wrapper">
                {/* Day labels */}
                <div className="heatmap-days">
                  <div className="day-label"></div>
                  <div className="day-label">Mon</div>
                  <div className="day-label"></div>
                  <div className="day-label">Wed</div>
                  <div className="day-label"></div>
                  <div className="day-label">Fri</div>
                  <div className="day-label"></div>
                </div>

                {/* Heatmap grid */}
                <div className="heatmap-grid">
                  {heatmapData.weeks.map((week, weekIndex) => (
                    <div key={weekIndex} className="heatmap-week">
                      {week.map((day, dayIndex) => (
                        <div
                          key={dayIndex}
                          className={`heatmap-day level-${day.level}`}
                        />
                      ))}
                    </div>
                  ))}
                </div>
              </div>

              {/* Legend */}
              <div className="heatmap-legend">
                <span className="legend-label">Less</span>
                <div className="legend-square level-0"></div>
                <div className="legend-square level-1"></div>
                <div className="legend-square level-2"></div>
                <div className="legend-square level-3"></div>
                <div className="legend-square level-4"></div>
                <span className="legend-label">More</span>
              </div>
            </div>
          </div>
        </section>

        <section className="features-section">
          <h3>Features</h3>
          <div className="features-list">
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Multi-Platform Support</span>
            </div>
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Privacy Controls</span>
            </div>
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Customizable Themes</span>
            </div>
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Automatic Sync</span>
            </div>
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Embeddable Heatmaps</span>
            </div>
            <div className="feature-item">
              <span className="feature-bullet">✓</span>
              <span>Activity Timeline</span>
            </div>
          </div>
        </section>

        <section className="how-it-works-section">
          <h3>How It Works</h3>
          <div className="steps-simple">
            <div className="step-simple">1. Create Account</div>
            <div className="step-simple">2. Connect Platforms</div>
            <div className="step-simple">3. View Your Heatmap</div>
          </div>
        </section>

      </main>

      <footer className="landing-footer">
        <div className="footer-content">
          <div className="footer-logo">
            <HeatmapLogo size={24} />
            <span>Hgitmap</span>
          </div>
          <p className="footer-text">
            Unified contribution tracking across GitHub, GitLab, and Gitea
          </p>
        </div>
      </footer>
    </div>
  );
};

export default Landing;
