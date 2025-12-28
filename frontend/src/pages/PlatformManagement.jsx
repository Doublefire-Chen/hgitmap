import { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import PlatformConnector from '../components/PlatformConnector';
import OAuthAppsManager from '../components/OAuthAppsManager';
import './PlatformManagement.css';

export default function PlatformManagement() {
  const { user } = useAuth();
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState('platforms');

  useEffect(() => {
    const tab = searchParams.get('tab');
    if (tab === 'oauth-apps' && user?.is_admin) {
      setActiveTab('oauth-apps');
    } else {
      setActiveTab('platforms');
    }
  }, [searchParams, user]);

  const handleTabChange = (tab) => {
    setActiveTab(tab);
    setSearchParams({ tab });
  };

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
            onClick={() => handleTabChange('platforms')}
          >
            My Platforms
          </button>
          <button
            className={`tab ${activeTab === 'oauth-apps' ? 'active' : ''}`}
            onClick={() => handleTabChange('oauth-apps')}
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
