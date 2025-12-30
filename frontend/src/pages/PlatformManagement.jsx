import { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import PlatformConnector from '../components/PlatformConnector';
import OAuthAppsManager from '../components/OAuthAppsManager';
import SyncJobsViewer from '../components/SyncJobsViewer';
import './PlatformManagement.css';

export default function PlatformManagement() {
  const { user } = useAuth();
  const [searchParams, setSearchParams] = useSearchParams();

  // Initialize activeTab from URL params
  const initTab = () => {
    const tab = searchParams.get('tab');
    if (tab === 'sync-jobs') {
      return 'sync-jobs';
    }
    if (tab === 'oauth-apps' && user?.is_admin) {
      return 'oauth-apps';
    }
    return 'platforms';
  };

  const [activeTab, setActiveTab] = useState(initTab);

  // Update tab when URL changes
  useEffect(() => {
    const tab = searchParams.get('tab');
    let newTab = 'platforms';

    if (tab === 'sync-jobs') {
      newTab = 'sync-jobs';
    } else if (tab === 'oauth-apps' && user?.is_admin) {
      newTab = 'oauth-apps';
    }

    if (newTab !== activeTab) {
      setActiveTab(newTab);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
          Manage your connected Git platforms and sync jobs{user?.is_admin && ', plus OAuth applications'}
        </p>
      </div>

      <div className="tabs">
        <button
          className={`tab ${activeTab === 'platforms' ? 'active' : ''}`}
          onClick={() => handleTabChange('platforms')}
        >
          My Platforms
        </button>
        <button
          className={`tab ${activeTab === 'sync-jobs' ? 'active' : ''}`}
          onClick={() => handleTabChange('sync-jobs')}
        >
          Sync Jobs
        </button>
        {user?.is_admin && (
          <button
            className={`tab ${activeTab === 'oauth-apps' ? 'active' : ''}`}
            onClick={() => handleTabChange('oauth-apps')}
          >
            OAuth Apps
            <span className="admin-badge">Admin</span>
          </button>
        )}
      </div>

      <div className="tab-content">
        {activeTab === 'platforms' && <PlatformConnector />}
        {activeTab === 'sync-jobs' && <SyncJobsViewer />}
        {activeTab === 'oauth-apps' && user?.is_admin && <OAuthAppsManager />}
      </div>
    </div>
  );
}
