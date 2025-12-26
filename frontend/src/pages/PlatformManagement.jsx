import { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import PlatformConnector from '../components/PlatformConnector';
import OAuthAppsManager from '../components/OAuthAppsManager';
import './PlatformManagement.css';

export default function PlatformManagement() {
  const { user } = useAuth();
  const [activeTab, setActiveTab] = useState('platforms');

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
