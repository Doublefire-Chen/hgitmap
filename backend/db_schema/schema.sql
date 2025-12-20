-- Database Schema for hgitmap
-- PostgreSQL 14+

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    is_admin BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Git platform type enum
CREATE TYPE git_platform AS ENUM ('github', 'gitea', 'gitlab');

-- Git platform accounts table
CREATE TABLE git_platform_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform_type git_platform NOT NULL,
    platform_username VARCHAR(255) NOT NULL,
    access_token TEXT, -- Encrypted OAuth token or API key
    refresh_token TEXT, -- For OAuth refresh
    platform_url VARCHAR(512), -- For self-hosted instances (Gitea, GitLab)
    is_active BOOLEAN DEFAULT true,
    last_synced_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, platform_type, platform_username, platform_url)
);

-- Contributions table
CREATE TABLE contributions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    git_platform_account_id UUID NOT NULL REFERENCES git_platform_accounts(id) ON DELETE CASCADE,
    contribution_date DATE NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    repository_name VARCHAR(512),
    is_private_repo BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast date-range queries
CREATE INDEX idx_contributions_date ON contributions(contribution_date);
CREATE INDEX idx_contributions_account_date ON contributions(git_platform_account_id, contribution_date);

-- User settings table
CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    show_private_contributions BOOLEAN DEFAULT true,
    hide_private_repo_names BOOLEAN DEFAULT false,
    heatmap_color_scheme VARCHAR(50) DEFAULT 'green',
    heatmap_size VARCHAR(20) DEFAULT 'medium',
    dark_mode_enabled BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- API tokens for embedding heatmap images
CREATE TABLE api_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_api_tokens_token ON api_tokens(token);

-- OAuth applications table (for web-based OAuth configuration)
CREATE TABLE oauth_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    platform git_platform NOT NULL,
    instance_url VARCHAR(512) NOT NULL DEFAULT '',
    instance_name VARCHAR(255) NOT NULL,
    client_id VARCHAR(512) NOT NULL,
    client_secret TEXT NOT NULL, -- Encrypted with AES-256-GCM
    is_enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(platform, instance_url)
);

-- Index for OAuth app lookups
CREATE INDEX idx_oauth_apps_platform ON oauth_applications(platform, instance_url, is_enabled);

-- OAuth state table for secure callback handling
CREATE TABLE oauth_states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    state_token VARCHAR(255) UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform git_platform NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_oauth_states_token ON oauth_states(state_token);
CREATE INDEX idx_oauth_states_expires ON oauth_states(expires_at);

-- Trigger function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_git_platform_accounts_updated_at BEFORE UPDATE ON git_platform_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_contributions_updated_at BEFORE UPDATE ON contributions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_settings_updated_at BEFORE UPDATE ON user_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_oauth_applications_updated_at BEFORE UPDATE ON oauth_applications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
