import { Link } from 'react-router-dom';
import HeatmapLogo from '../components/HeatmapLogo';
import ThemeToggle from '../components/ThemeToggle';
import './Landing.css';

const Landing = () => {
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
              <div className="preview-grid">
                {Array.from({ length: 52 * 7 }).map((_, idx) => {
                  const level = Math.floor(Math.random() * 5);
                  return (
                    <div
                      key={idx}
                      className={`preview-cell level-${level}`}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        </section>

        <section className="features-section">
          <h3>Features</h3>
          <div className="features-grid">
            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M8.75 1.75a.75.75 0 0 0-1.5 0V5H4a.75.75 0 0 0 0 1.5h3.25v3.25H4a.75.75 0 0 0 0 1.5h3.25V15a.75.75 0 0 0 1.5 0v-3.75H12a.75.75 0 0 0 0-1.5H8.75V6.5H12a.75.75 0 0 0 0-1.5H8.75V1.75z"></path>
                </svg>
              </div>
              <h4>Multi-Platform Support</h4>
              <p>Connect GitHub, GitLab, and Gitea accounts. Support for self-hosted instances included.</p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M8 0a8.2 8.2 0 0 1 .701.031C9.444.095 9.99.645 10.16 1.29l.288 1.107c.018.066.079.158.212.224.231.114.454.243.668.386.123.082.233.09.299.071l1.103-.303c.644-.176 1.392.021 1.82.63.27.385.506.792.704 1.218.315.675.111 1.422-.364 1.891l-.814.806c-.049.048-.098.147-.088.294.016.257.016.515 0 .772-.01.147.038.246.088.294l.814.806c.475.469.679 1.216.364 1.891a7.977 7.977 0 0 1-.704 1.217c-.428.61-1.176.807-1.82.63l-1.102-.302c-.067-.019-.177-.011-.3.071a5.909 5.909 0 0 1-.668.386c-.133.066-.194.158-.211.224l-.29 1.106c-.168.646-.715 1.196-1.458 1.26a8.006 8.006 0 0 1-1.402 0c-.743-.064-1.289-.614-1.458-1.26l-.289-1.106c-.018-.066-.079-.158-.212-.224a5.738 5.738 0 0 1-.668-.386c-.123-.082-.233-.09-.299-.071l-1.103.303c-.644.176-1.392-.021-1.82-.63a8.12 8.12 0 0 1-.704-1.218c-.315-.675-.111-1.422.363-1.891l.815-.806c.05-.048.098-.147.088-.294a6.214 6.214 0 0 1 0-.772c.01-.147-.038-.246-.088-.294l-.815-.806C.635 6.045.431 5.298.746 4.623a7.92 7.92 0 0 1 .704-1.217c.428-.61 1.176-.807 1.82-.63l1.102.302c.067.019.177.011.3-.071.214-.143.437-.272.668-.386.133-.066.194-.158.211-.224l.29-1.106C6.009.645 6.556.095 7.299.03 7.53.01 7.764 0 8 0Zm-.571 1.525c-.036.003-.108.036-.137.146l-.289 1.105c-.147.561-.549.967-.998 1.189-.173.086-.34.183-.5.29-.417.278-.97.423-1.529.27l-1.103-.303c-.109-.03-.175.016-.195.045-.22.312-.412.644-.573.99-.014.031-.021.11.059.19l.815.806c.411.406.562.957.53 1.456a4.709 4.709 0 0 0 0 .582c.032.499-.119 1.05-.53 1.456l-.815.806c-.081.08-.073.159-.059.19.162.346.353.677.573.989.02.03.085.076.195.046l1.102-.303c.56-.153 1.113-.008 1.53.27.161.107.328.204.501.29.447.222.85.629.997 1.189l.289 1.105c.029.109.101.143.137.146a6.6 6.6 0 0 0 1.142 0c.036-.003.108-.036.137-.146l.289-1.105c.147-.561.549-.967.998-1.189.173-.086.34-.183.5-.29.417-.278.97-.423 1.529-.27l1.103.303c.109.029.175-.016.195-.045.22-.313.411-.644.573-.99.014-.031.021-.11-.059-.19l-.815-.806c-.411-.406-.562-.957-.53-1.456a4.709 4.709 0 0 0 0-.582c-.032-.499.119-1.05.53-1.456l.815-.806c.081-.08.073-.159.059-.19a6.464 6.464 0 0 0-.573-.989c-.02-.03-.085-.076-.195-.046l-1.102.303c-.56.153-1.113.008-1.53-.27a4.44 4.44 0 0 0-.501-.29c-.447-.222-.85-.629-.997-1.189l-.289-1.105c-.029-.11-.101-.143-.137-.146a6.6 6.6 0 0 0-1.142 0ZM11 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM9.5 8a1.5 1.5 0 1 0-3.001.001A1.5 1.5 0 0 0 9.5 8Z"></path>
                </svg>
              </div>
              <h4>Privacy Controls</h4>
              <p>Control whether private repository contributions are displayed and whether repo names are visible.</p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z"></path>
                </svg>
              </div>
              <h4>Customizable Themes</h4>
              <p>Create and manage custom heatmap themes with your preferred colors and styles.</p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M1.705 8.005a.75.75 0 0 1 .834.656 5.5 5.5 0 0 0 9.592 2.97l-1.204-1.204a.25.25 0 0 1 .177-.427h3.646a.25.25 0 0 1 .25.25v3.646a.25.25 0 0 1-.427.177l-1.38-1.38A7.002 7.002 0 0 1 1.05 8.84a.75.75 0 0 1 .656-.834ZM8 2.5a5.487 5.487 0 0 0-4.131 1.869l1.204 1.204A.25.25 0 0 1 4.896 6H1.25A.25.25 0 0 1 1 5.75V2.104a.25.25 0 0 1 .427-.177l1.38 1.38A7.002 7.002 0 0 1 14.95 7.16a.75.75 0 0 1-1.49.178A5.5 5.5 0 0 0 8 2.5Z"></path>
                </svg>
              </div>
              <h4>Automatic Sync</h4>
              <p>Keep your heatmap up-to-date with automated background syncing of contribution data.</p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v12.5A1.75 1.75 0 0 1 14.25 16H1.75A1.75 1.75 0 0 1 0 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25ZM9.5 5.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM4.75 11.5a.75.75 0 0 0 0 1.5h6.5a.75.75 0 0 0 0-1.5h-6.5Z"></path>
                </svg>
              </div>
              <h4>Embeddable Heatmaps</h4>
              <p>Generate shareable heatmap images to embed in your README, portfolio, or website.</p>
            </div>

            <div className="feature-card">
              <div className="feature-icon">
                <svg viewBox="0 0 16 16" width="24" height="24" fill="currentColor">
                  <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.249.249 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z"></path>
                </svg>
              </div>
              <h4>Activity Timeline</h4>
              <p>View detailed activity timeline with commits, pull requests, and contributions across platforms.</p>
            </div>
          </div>
        </section>

        <section className="how-it-works-section">
          <h3>How It Works</h3>
          <div className="steps-container">
            <div className="step">
              <div className="step-number">1</div>
              <h4>Create Account</h4>
              <p>Sign up for a free account with just a username and password.</p>
            </div>
            <div className="step-arrow">→</div>
            <div className="step">
              <div className="step-number">2</div>
              <h4>Connect Platforms</h4>
              <p>Link your GitHub, GitLab, and Gitea accounts via OAuth or access tokens.</p>
            </div>
            <div className="step-arrow">→</div>
            <div className="step">
              <div className="step-number">3</div>
              <h4>Sync & View</h4>
              <p>Your contributions are aggregated and displayed in a unified heatmap.</p>
            </div>
          </div>
        </section>

        <section className="cta-section">
          <h3>Ready to visualize your contributions?</h3>
          <p>Join developers tracking their coding activity across all platforms.</p>
          <Link to="/register" className="btn-primary btn-large">
            Get Started Now
          </Link>
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
