import { useState, useEffect } from 'react';
import apiClient from '../api/client';
import './SyncJobStatus.css';

function SyncJobStatus({ platformId, refreshTrigger }) {
    const [jobs, setJobs] = useState([]);
    const [loading, setLoading] = useState(true);

    const loadJobs = async () => {
        try {
            // Get recent sync jobs for this platform
            const allJobs = await apiClient.listPlatformSyncJobs();

            // Filter to only show jobs for this platform
            const platformJobs = allJobs
                .filter(job => job.platform_account_id === platformId)
                .slice(0, 5); // Show last 5 jobs

            setJobs(platformJobs);
            setLoading(false);
        } catch (err) {
            console.error('Failed to load sync jobs:', err);
            setLoading(false);
        }
    };

    useEffect(() => {
        loadJobs();

        // Poll for job updates every 5 seconds
        const interval = setInterval(loadJobs, 5000);

        return () => clearInterval(interval);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [platformId, refreshTrigger]);

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
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;

        const diffHours = Math.floor(diffMins / 60);
        if (diffHours < 24) return `${diffHours}h ago`;

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

    const renderProgressBar = (job) => {
        if (job.status !== 'processing' && job.status !== 'completed') {
            return null;
        }

        const contributions = job.contributions_synced || 0;
        const activities = job.activities_synced || 0;
        const total = contributions + activities;

        if (total === 0 && job.status === 'processing') {
            // Show indeterminate progress bar when starting
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
                    <div className="progress-bar animated">
                        <div className="progress-bar-fill"></div>
                    </div>
                )}
            </div>
        );
    };

    if (loading) {
        return null; // Don't show anything while loading
    }

    if (jobs.length === 0) {
        return null; // Don't show component if no jobs
    }

    return (
        <div className="sync-job-status">
            <h4>Recent Sync Jobs</h4>
            <div className="jobs-list">
                {jobs.map((job) => (
                    <div key={job.id} className="job-item">
                        <div className="job-header">
                            <span className={`status-badge ${getStatusBadgeClass(job.status)}`}>
                                {job.status}
                            </span>
                            <span className="job-time">{formatDate(job.scheduled_at)}</span>
                        </div>
                        <div className="job-description">
                            {getSyncDescription(job)}
                        </div>
                        {renderProgressBar(job)}
                        {job.error_message && (
                            <div className="job-error">
                                Error: {job.error_message}
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

export default SyncJobStatus;
