import { useState, useEffect } from 'react';
import apiClient from '../api/client';
import PlatformIcon from './PlatformIcon';
import './SyncJobsViewer.css';

function SyncJobsViewer() {
    const [jobs, setJobs] = useState([]);
    const [platforms, setPlatforms] = useState([]);
    const [loading, setLoading] = useState(true);
    const [statusFilter, setStatusFilter] = useState('all');
    const [platformFilter, setPlatformFilter] = useState('all');

    const loadJobs = async () => {
        try {
            const allJobs = await apiClient.listPlatformSyncJobs();
            setJobs(allJobs);
            setLoading(false);
        } catch (err) {
            console.error('Failed to load sync jobs:', err);
            setLoading(false);
        }
    };

    const loadPlatforms = async () => {
        try {
            const platformsData = await apiClient.listPlatforms();
            setPlatforms(platformsData);
        } catch (err) {
            console.error('Failed to load platforms:', err);
        }
    };

    const handleCancelJob = async (jobId) => {
        if (!confirm('Are you sure you want to cancel this sync job?')) {
            return;
        }

        try {
            await apiClient.cancelSyncJob(jobId);
            await loadJobs(); // Refresh the list
        } catch (err) {
            console.error('Failed to cancel sync job:', err);
            alert('Failed to cancel sync job');
        }
    };

    const handleDeleteJob = async (jobId) => {
        if (!confirm('Are you sure you want to delete this sync job?')) {
            return;
        }

        try {
            await apiClient.deleteSyncJob(jobId);
            await loadJobs(); // Refresh the list
        } catch (err) {
            console.error('Failed to delete sync job:', err);
            alert('Failed to delete sync job');
        }
    };

    useEffect(() => {
        const fetchData = async () => {
            await Promise.all([loadJobs(), loadPlatforms()]);
        };

        fetchData();

        // Poll for job updates every 5 seconds
        const interval = setInterval(loadJobs, 5000);

        return () => clearInterval(interval);
    }, []);

    const getStatusBadgeClass = (status) => {
        switch (status) {
            case 'completed':
                return 'status-completed';
            case 'processing':
                return 'status-processing';
            case 'failed':
                return 'status-failed';
            default:
                return 'status-pending';
        }
    };

    const formatDate = (dateString) => {
        if (!dateString) return 'N/A';
        const date = new Date(dateString);
        return date.toLocaleString();
    };

    const formatRelativeTime = (dateString) => {
        if (!dateString) return 'N/A';
        const date = new Date(dateString);
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;

        const diffHours = Math.floor(diffMins / 60);
        if (diffHours < 24) return `${diffHours}h ago`;

        const diffDays = Math.floor(diffHours / 24);
        if (diffDays < 7) return `${diffDays}d ago`;

        return date.toLocaleDateString();
    };

    const getSyncDescription = (job) => {
        const parts = [];

        if (job.sync_all_years) {
            parts.push('All years');
        } else if (job.specific_year) {
            parts.push(`Year ${job.specific_year}`);
        } else {
            parts.push('Current year');
        }

        const syncTypes = [];
        if (job.sync_contributions) syncTypes.push('contributions');
        if (job.sync_activities) syncTypes.push('activities');
        if (job.sync_profile) syncTypes.push('profile');

        if (syncTypes.length > 0) {
            parts.push(`(${syncTypes.join(', ')})`);
        }

        return parts.join(' ');
    };

    const getPlatformInfo = (platformAccountId) => {
        return platforms.find(p => p.id === platformAccountId);
    };

    const renderProgressBar = (job) => {
        if (job.status !== 'processing' && job.status !== 'completed') {
            return null;
        }

        const contributions = job.contributions_synced || 0;
        const activities = job.activities_synced || 0;
        const total = contributions + activities;
        const yearsCompleted = job.years_completed || 0;
        const totalYears = job.total_years || 0;

        // Calculate percentage based on years
        const percentage = totalYears > 0 ? Math.round((yearsCompleted / totalYears) * 100) : 0;

        if (total === 0 && job.status === 'processing') {
            return (
                <div className="progress-container">
                    <div className="progress-bar indeterminate">
                        <div className="progress-bar-fill"></div>
                    </div>
                    <div className="progress-text">Initializing...</div>
                </div>
            );
        }

        if (total === 0) {
            return null;
        }

        return (
            <div className="progress-container">
                <div className="progress-counts">
                    <span className="count-item">
                        <span className="count-number">{contributions}</span> contributions
                    </span>
                    <span className="count-divider">•</span>
                    <span className="count-item">
                        <span className="count-number">{activities}</span> activities
                    </span>
                </div>
                {job.status === 'processing' && (
                    <div className="progress-bar-wrapper">
                        <div className="progress-bar">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${percentage}%` }}
                            ></div>
                        </div>
                        <div className="progress-percentage">{percentage}%</div>
                    </div>
                )}
            </div>
        );
    };

    const filteredJobs = jobs.filter(job => {
        if (statusFilter !== 'all' && job.status !== statusFilter) {
            return false;
        }
        if (platformFilter !== 'all' && job.platform_account_id !== platformFilter) {
            return false;
        }
        return true;
    });

    if (loading) {
        return (
            <div className="sync-jobs-viewer">
                <div className="loading">Loading sync jobs...</div>
            </div>
        );
    }

    return (
        <div className="sync-jobs-viewer">
            <div className="jobs-header">
                <div className="jobs-stats">
                    <h2>Sync Jobs</h2>
                    <p className="jobs-count">{filteredJobs.length} job{filteredJobs.length !== 1 ? 's' : ''}</p>
                </div>

                <div className="jobs-filters">
                    <select
                        value={statusFilter}
                        onChange={(e) => setStatusFilter(e.target.value)}
                        className="filter-select"
                    >
                        <option value="all">All Status</option>
                        <option value="pending">Pending</option>
                        <option value="processing">Processing</option>
                        <option value="completed">Completed</option>
                        <option value="failed">Failed</option>
                    </select>

                    <select
                        value={platformFilter}
                        onChange={(e) => setPlatformFilter(e.target.value)}
                        className="filter-select"
                    >
                        <option value="all">All Platforms</option>
                        {platforms.map(platform => (
                            <option key={platform.id} value={platform.id}>
                                {platform.platform_username} ({platform.platform})
                            </option>
                        ))}
                    </select>
                </div>
            </div>

            {filteredJobs.length === 0 ? (
                <div className="empty-state">
                    <p>No sync jobs found.</p>
                    <p className="empty-hint">Start a sync from the Platforms tab to see jobs here.</p>
                </div>
            ) : (
                <div className="jobs-list">
                    {filteredJobs.map((job) => {
                        const platform = getPlatformInfo(job.platform_account_id);
                        return (
                            <div key={job.id} className="job-card">
                                <div className="job-card-header">
                                    <div className="platform-info">
                                        {platform && (
                                            <>
                                                <PlatformIcon platform={platform.platform} size={20} />
                                                <span className="platform-name">{platform.platform_username}</span>
                                                <span className="platform-type">{platform.platform}</span>
                                            </>
                                        )}
                                    </div>
                                    <div className="job-status-time">
                                        <span className={`status-badge ${getStatusBadgeClass(job.status)}`}>
                                            {job.status}
                                        </span>
                                        <span className="job-time" title={formatDate(job.scheduled_at)}>
                                            {formatRelativeTime(job.scheduled_at)}
                                        </span>
                                    </div>
                                </div>

                                <div className="job-card-body">
                                    <div className="job-description">
                                        {getSyncDescription(job)}
                                    </div>

                                    {renderProgressBar(job)}

                                    {job.error_message && (
                                        <div className="job-error">
                                            <strong>Error:</strong> {job.error_message}
                                        </div>
                                    )}

                                    {job.retry_count > 0 && (
                                        <div className="job-retry">
                                            Retry attempt {job.retry_count}/{job.max_retries}
                                        </div>
                                    )}
                                </div>

                                <div className="job-card-actions">
                                    {(job.status === 'pending' || job.status === 'processing') && (
                                        <button
                                            className="btn-cancel"
                                            onClick={() => handleCancelJob(job.id)}
                                            title="Cancel this sync job"
                                        >
                                            Cancel
                                        </button>
                                    )}
                                    {(job.status === 'completed' || job.status === 'failed') && (
                                        <button
                                            className="btn-delete"
                                            onClick={() => handleDeleteJob(job.id)}
                                            title="Delete this sync job"
                                        >
                                            Delete
                                        </button>
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}

export default SyncJobsViewer;
