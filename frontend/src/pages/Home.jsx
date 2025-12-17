import { useAuth } from '../context/AuthContext';
import { useNavigate } from 'react-router-dom';
import './Home.css';

const Home = () => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="home-container">
      <header className="home-header">
        <div className="header-content">
          <h1>hgitmap</h1>
          <div className="user-info">
            <span>Welcome, {user?.username}!</span>
            <button onClick={handleLogout} className="logout-btn">
              Logout
            </button>
          </div>
        </div>
      </header>

      <main className="home-content">
        <div className="welcome-section">
          <h2>Your Contribution Heatmap</h2>
          <p>Connect your git platform accounts to see your unified contribution history.</p>

          <div className="placeholder-section">
            <p className="placeholder-text">
              🚧 Coming soon: Connect GitHub, GitLab, and Gitea accounts
            </p>
          </div>
        </div>
      </main>
    </div>
  );
};

export default Home;
