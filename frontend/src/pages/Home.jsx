import { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { useNavigate, Link } from 'react-router-dom';
import ThemeToggle from '../components/ThemeToggle';
import Heatmap from '../components/Heatmap';
import PlatformConnector from '../components/PlatformConnector';
import ActivityTimeline from '../components/ActivityTimeline';
import HeatmapLogo from '../components/HeatmapLogo';
import './Home.css';

const Home = () => {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState('overview');

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="home-container">
      <header className="home-header">
        <div className="header-content">
          <div className="logo-section">
            <HeatmapLogo size={32} />
            <h1>Hgitmap</h1>
          </div>
          <div className="user-info">
            <span className="welcome-text">Welcome, {user?.username}!</span>
            <ThemeToggle />
            <Link to="/settings" className="settings-btn">
              Settings
            </Link>
            <button onClick={handleLogout} className="logout-btn">
              Logout
            </button>
          </div>
        </div>

        <div className="tabs">
          <button
            className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
            onClick={() => setActiveTab('overview')}
          >
            Overview
          </button>
          <button
            className={`tab ${activeTab === 'platforms' ? 'active' : ''}`}
            onClick={() => setActiveTab('platforms')}
          >
            Platforms
          </button>
        </div>
      </header>

      <main className="home-content">
        {activeTab === 'overview' && (
          <>
            <Heatmap />
            <ActivityTimeline />
          </>
        )}
        {activeTab === 'platforms' && <PlatformConnector />}
      </main>
    </div>
  );
};

export default Home;
