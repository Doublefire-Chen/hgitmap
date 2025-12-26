import { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { useLocation } from 'react-router-dom';
import PlatformConnector from '../components/PlatformConnector';
import OAuthAppsManager from '../components/OAuthAppsManager';
import './PlatformManagement.css';

export default function PlatformManagement() {
  const { user } = useAuth();
  const location = useLocation();

  // Default to 'oauth-apps' tab if user is admin and came from /settings/oauth-apps route
  const [activeTab, setActiveTab] = useState(() => {
    if (user?.is_admin && location.pathname.includes('oauth-apps')) {
      return 'oauth-apps';
    }
    return 'platforms';
  });

  // Update tab when location changes
  useEffect(() => {
    if (location.pathname.includes('oauth-apps') && user?.is_admin) {
      setActiveTab('oauth-apps');
    } else if (location.pathname.includes('platforms')) {
      setActiveTab('platforms');
    }
  }, [location.pathname, user?.is_admin]);

  return (
    <div className="platform-management">
      <div className="platform-management-header">
        <h1>Platform Management</h1>
        <p className="subtitle">
          Manage your connected Git platforms {user?.is_admin && 'and OAuth applications'}
        </p>
      </div>

      {user?.is_admin && (
        <div className="tabs">
          <button
            className={`tab ${activeTab === 'platforms' ? 'active' : ''}`}
            onClick={() => setActiveTab('platforms')}
          >
            My Platforms
          </button>
          <button
            className={`tab ${activeTab === 'oauth-apps' ? 'active' : ''}`}
            onClick={() => setActiveTab('oauth-apps')}
          >
            OAuth Apps
            <span className="admin-badge">Admin</span>
          </button>
        </div>
      )}

      <div className="tab-content">
        {activeTab === 'platforms' && <PlatformConnector />}
        {activeTab === 'oauth-apps' && user?.is_admin && <OAuthAppsManager />}
      </div>
    </div>
  );
}
