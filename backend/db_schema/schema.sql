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

-- Authentication type enum
CREATE TYPE auth_type AS ENUM ('oauth', 'personal_access_token');

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
    -- Profile fields from the git platform
    avatar_url VARCHAR(1024),
    display_name VARCHAR(255),
    bio TEXT,
    profile_url VARCHAR(1024),
    location VARCHAR(255),
    company VARCHAR(255),
    followers_count INTEGER DEFAULT 0,
    following_count INTEGER DEFAULT 0,
    -- Sync preferences (per-platform control)
    sync_profile BOOLEAN DEFAULT true, -- Enable/disable syncing profile data (avatar, bio, etc.)
    sync_contributions BOOLEAN DEFAULT true, -- Enable/disable syncing contributions and activities together
    -- Authentication method
    auth_type auth_type NOT NULL DEFAULT 'oauth',
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
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    -- Unique constraint: one contribution per account, date, and repository
    -- NULLS NOT DISTINCT ensures only one NULL repository_name per date
    CONSTRAINT unique_contribution_per_account_date_repo
        UNIQUE NULLS NOT DISTINCT (git_platform_account_id, contribution_date, repository_name)
);

-- Index for fast date-range queries
CREATE INDEX idx_contributions_date ON contributions(contribution_date);
CREATE INDEX idx_contributions_account_date ON contributions(git_platform_account_id, contribution_date);

-- Activity type enum for contribution timeline
CREATE TYPE activity_type AS ENUM (
    'commit',
    'repository_created',
    'pull_request',
    'issue',
    'review',
    'organization_joined',
    'fork',
    'release',
    'star'
);

-- Activities table for contribution timeline
-- This stores various types of activities (commits, repos created, PRs, issues, etc.)
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    git_platform_account_id UUID NOT NULL REFERENCES git_platform_accounts(id) ON DELETE CASCADE,
    activity_type activity_type NOT NULL,
    activity_date DATE NOT NULL,

    -- Activity metadata (stored as JSONB for flexibility)
    -- Structure varies by activity_type (see examples below)
    metadata JSONB NOT NULL DEFAULT '{}',

    -- Common fields extracted for easier querying
    repository_name VARCHAR(512),
    repository_url VARCHAR(1024),
    is_private_repo BOOLEAN DEFAULT false,
    count INTEGER DEFAULT 1, -- For commits, this is the commit count

    -- Primary language for repository_created activities
    primary_language VARCHAR(50),

    -- Organization name for organization_joined activities
    organization_name VARCHAR(255),
    organization_avatar_url VARCHAR(1024),

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for fast activity queries
CREATE INDEX idx_activities_account_date ON activities(git_platform_account_id, activity_date DESC);
CREATE INDEX idx_activities_date ON activities(activity_date DESC);
CREATE INDEX idx_activities_type ON activities(activity_type);
CREATE INDEX idx_activities_metadata ON activities USING GIN (metadata);

-- Example metadata structures for different activity types:
--
-- commit:
-- {
--   "repositories": [
--     {
--       "name": "username/repo",
--       "commit_count": 10,
--       "commits": [{"sha": "abc123", "message": "...", "url": "..."}]
--     }
--   ],
--   "total_count": 12
-- }
--
-- repository_created:
-- {
--   "name": "username/repo",
--   "description": "...",
--   "language": "Rust",
--   "is_fork": false,
--   "created_at": "2025-12-16T00:00:00Z"
-- }
--
-- organization_joined:
-- {
--   "organization": "org-name",
--   "avatar_url": "https://...",
--   "joined_at": "2025-12-14T00:00:00Z"
-- }
--
-- pull_request/issue:
-- {
--   "title": "Feature X",
--   "number": 123,
--   "state": "open/closed/merged",
--   "repository": "owner/repo",
--   "url": "https://..."
-- }

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
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    instance_url VARCHAR(500) -- Instance URL for self-hosted platforms (Gitea/GitLab OAuth)
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

CREATE TRIGGER update_activities_updated_at BEFORE UPDATE ON activities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_settings_updated_at BEFORE UPDATE ON user_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_oauth_applications_updated_at BEFORE UPDATE ON oauth_applications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- Heatmap Theme and Generation System Schema
-- ============================================================

-- Heatmap color scheme presets
CREATE TYPE heatmap_color_scheme AS ENUM (
    'github_green',
    'github_blue',
    'halloween',
    'winter',
    'ocean',
    'sunset',
    'forest',
    'monochrome',
    'rainbow',
    'custom'
);

-- Theme mode (light/dark)
CREATE TYPE theme_mode AS ENUM ('light', 'dark');

-- Output format for heatmaps
CREATE TYPE heatmap_format AS ENUM ('svg', 'png', 'jpeg', 'webp');

-- Heatmap themes table
-- Users can define multiple themes with different color schemes, sizes, and styles
CREATE TABLE heatmap_themes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Theme identification
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL, -- URL-safe identifier (e.g., 'dark-ocean', 'light-green')
    description TEXT,
    is_default BOOLEAN DEFAULT false,

    -- Light/Dark mode
    theme_mode theme_mode NOT NULL DEFAULT 'light',

    -- Color configuration
    color_scheme heatmap_color_scheme NOT NULL DEFAULT 'github_green',

    -- Custom contribution level colors (for 'custom' scheme type)
    -- Stored as JSON array of 5 colors from low to high intensity
    -- Example: ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]
    custom_colors JSONB,

    -- Theme colors
    background_color VARCHAR(7) DEFAULT '#ffffff', -- Hex color
    border_color VARCHAR(7) DEFAULT '#d1d5da',
    text_color VARCHAR(7) DEFAULT '#24292e',
    empty_cell_color VARCHAR(7) DEFAULT '#ebedf0', -- Color for days with no contributions

    -- Cell/Rectangle styling
    cell_size INTEGER DEFAULT 10, -- Size of each cell in pixels (width and height)
    cell_gap INTEGER DEFAULT 2, -- Gap between cells in pixels
    cell_border_radius INTEGER DEFAULT 2, -- Border radius for rounded corners (0 = square)
    cell_border_width INTEGER DEFAULT 0, -- Border width for cells (0 = no border)
    cell_border_color VARCHAR(7) DEFAULT '#d1d5da',

    -- Overall heatmap dimensions
    -- If null, auto-calculate based on data and cell size
    heatmap_width INTEGER, -- Total width in pixels
    heatmap_height INTEGER, -- Total height in pixels

    -- Padding around the heatmap
    padding_top INTEGER DEFAULT 20,
    padding_right INTEGER DEFAULT 20,
    padding_bottom INTEGER DEFAULT 17,
    padding_left INTEGER DEFAULT 20,

    -- Layout spacing settings (customize spacing for various UI elements)
    day_label_width INTEGER DEFAULT 28, -- Width reserved for day labels (Mon, Wed, Fri)
    month_label_height INTEGER DEFAULT 15, -- Height reserved for month labels
    title_height INTEGER DEFAULT 30, -- Height reserved for title/header area
    legend_height INTEGER DEFAULT 8, -- Height reserved for legend area

    -- Display options
    show_month_labels BOOLEAN DEFAULT true,
    show_day_labels BOOLEAN DEFAULT true,
    show_legend BOOLEAN DEFAULT true,
    show_total_count BOOLEAN DEFAULT true, -- Show total contribution count
    show_username BOOLEAN DEFAULT true, -- Show username at top
    show_watermark BOOLEAN DEFAULT true, -- Show "Powered by Hgitmap" watermark

    -- Font settings
    font_family VARCHAR(255) DEFAULT 'sans-serif',
    font_size INTEGER DEFAULT 10, -- Font size in pixels

    -- Legend settings
    legend_position VARCHAR(20) DEFAULT 'bottom', -- 'top', 'bottom', 'left', 'right', 'none'

    -- Output formats (stored as array)
    -- Users can generate multiple formats for the same theme
    -- Example: ['png', 'svg'] or ['png', 'webp', 'jpeg']
    output_formats heatmap_format[] DEFAULT ARRAY['png']::heatmap_format[],

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(user_id, slug)
);

-- Index for fast theme lookups
CREATE INDEX idx_heatmap_themes_user ON heatmap_themes(user_id);
CREATE INDEX idx_heatmap_themes_slug ON heatmap_themes(user_id, slug);

-- Heatmap generation settings per user
CREATE TABLE heatmap_generation_settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Update interval in minutes (default: 60 minutes = 1 hour)
    -- Common values: 15, 30, 60, 180, 360, 720, 1440 (24 hours)
    update_interval_minutes INTEGER NOT NULL DEFAULT 60,

    -- Allow users to pause automatic generation
    auto_generation_enabled BOOLEAN DEFAULT false,

    -- Date range for heatmap (in days, e.g., 365 for one year)
    date_range_days INTEGER DEFAULT 365,

    -- Whether to include private contributions
    include_private_contributions BOOLEAN DEFAULT true,

    -- Storage path customization (relative to static files directory)
    storage_path VARCHAR(512), -- Optional custom path (default: /static/heatmaps/{user_id}/)

    -- Last scheduled generation time
    last_scheduled_generation_at TIMESTAMP WITH TIME ZONE,
    next_scheduled_generation_at TIMESTAMP WITH TIME ZONE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Generated heatmaps tracking table
-- Tracks all generated heatmap files and their metadata
-- Each format for a theme gets its own row
CREATE TABLE generated_heatmaps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    theme_id UUID NOT NULL REFERENCES heatmap_themes(id) ON DELETE CASCADE,

    -- Format of this generated file
    format heatmap_format NOT NULL,

    -- File information
    file_path VARCHAR(1024) NOT NULL, -- Path to the generated file (e.g., /static/heatmaps/{user_id}/{theme_slug}.png)
    file_size_bytes BIGINT, -- File size in bytes
    file_hash VARCHAR(64), -- SHA-256 hash for cache invalidation

    -- Generation metadata
    generated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    generation_duration_ms INTEGER, -- How long it took to generate

    -- Data snapshot info
    contribution_count INTEGER DEFAULT 0, -- Total contributions in this heatmap
    date_range_start DATE NOT NULL,
    date_range_end DATE NOT NULL,

    -- Access tracking
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMP WITH TIME ZONE,

    -- Status
    is_valid BOOLEAN DEFAULT true, -- Mark as invalid when needs regeneration

    UNIQUE(user_id, theme_id, format)
);

-- Indexes for generated heatmaps
CREATE INDEX idx_generated_heatmaps_user ON generated_heatmaps(user_id);
CREATE INDEX idx_generated_heatmaps_theme ON generated_heatmaps(theme_id);
CREATE INDEX idx_generated_heatmaps_valid ON generated_heatmaps(is_valid);
CREATE INDEX idx_generated_heatmaps_generated_at ON generated_heatmaps(generated_at);

-- Heatmap generation queue/job table
-- Tracks pending and completed generation jobs
CREATE TYPE generation_job_status AS ENUM ('pending', 'processing', 'completed', 'failed');

CREATE TABLE heatmap_generation_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    theme_id UUID REFERENCES heatmap_themes(id) ON DELETE CASCADE, -- NULL means regenerate all themes

    status generation_job_status NOT NULL DEFAULT 'pending',

    -- Job scheduling
    scheduled_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,

    -- Result tracking
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,

    -- Job metadata
    is_manual BOOLEAN DEFAULT false, -- User triggered vs automatic
    priority INTEGER DEFAULT 0, -- Higher priority jobs run first

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for job queue management
CREATE INDEX idx_generation_jobs_status ON heatmap_generation_jobs(status, scheduled_at);
CREATE INDEX idx_generation_jobs_user ON heatmap_generation_jobs(user_id);
CREATE INDEX idx_generation_jobs_priority ON heatmap_generation_jobs(priority DESC, scheduled_at);

-- Add triggers for updated_at
CREATE TRIGGER update_heatmap_themes_updated_at BEFORE UPDATE ON heatmap_themes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_heatmap_generation_settings_updated_at BEFORE UPDATE ON heatmap_generation_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to invalidate generated heatmaps when new contributions are added
CREATE OR REPLACE FUNCTION invalidate_heatmaps_on_contribution()
RETURNS TRIGGER AS $$
BEGIN
    -- Mark all generated heatmaps for this user as invalid
    UPDATE generated_heatmaps
    SET is_valid = false
    WHERE user_id IN (
        SELECT user_id
        FROM git_platform_accounts
        WHERE id = NEW.git_platform_account_id
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to invalidate heatmaps when contributions change
CREATE TRIGGER invalidate_heatmaps_on_new_contribution
    AFTER INSERT OR UPDATE ON contributions
    FOR EACH ROW
    EXECUTE FUNCTION invalidate_heatmaps_on_contribution();

-- Function to automatically create default themes for new users
CREATE OR REPLACE FUNCTION create_default_heatmap_themes()
RETURNS TRIGGER AS $$
BEGIN
    -- Create default light theme (matches current frontend UI exactly)
    INSERT INTO heatmap_themes (
        user_id,
        name,
        slug,
        description,
        is_default,
        theme_mode,
        color_scheme,
        custom_colors,
        background_color,
        border_color,
        text_color,
        empty_cell_color,
        cell_size,
        cell_gap,
        cell_border_radius,
        cell_border_width,
        cell_border_color,
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
        day_label_width,
        month_label_height,
        title_height,
        legend_height,
        show_month_labels,
        show_day_labels,
        show_legend,
        show_total_count,
        show_username,
        show_watermark,
        font_family,
        font_size,
        output_formats
    ) VALUES (
        NEW.id,
        'Default Light',
        'default-light',
        'Classic GitHub contribution graph style (light mode)',
        true,
        'light',
        'custom',
        '["#eff2f5", "#aceebb", "#4ac26b", "#2da44e", "#116329"]'::jsonb,
        '#ffffff',
        '#e1e4e8',
        '#586069',
        '#eff2f5',
        10,
        3,
        2,
        1,
        '#e1e4e8',
        20,
        20,
        10,
        28,
        28,
        15,
        30,
        8,
        true,
        true,
        true,
        true,
        false,
        false,
        'sans-serif',
        10,
        ARRAY['png', 'svg']::heatmap_format[]
    );

    -- Create default dark theme (matches current frontend dark mode)
    INSERT INTO heatmap_themes (
        user_id,
        name,
        slug,
        description,
        is_default,
        theme_mode,
        color_scheme,
        custom_colors,
        background_color,
        border_color,
        text_color,
        empty_cell_color,
        cell_size,
        cell_gap,
        cell_border_radius,
        cell_border_width,
        cell_border_color,
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
        day_label_width,
        month_label_height,
        title_height,
        legend_height,
        show_month_labels,
        show_day_labels,
        show_legend,
        show_total_count,
        show_username,
        show_watermark,
        font_family,
        font_size,
        output_formats
    ) VALUES (
        NEW.id,
        'Default Dark',
        'default-dark',
        'GitHub contribution graph for dark mode',
        false,
        'dark',
        'custom',
        '["#151b23", "#033a16", "#196c2e", "#2ea043", "#56d364"]'::jsonb,
        '#0d1117',
        '#30363d',
        '#8b949e',
        '#151b23',
        10,
        3,
        2,
        1,
        '#30363d',
        20,
        20,
        10,
        28,
        28,
        15,
        30,
        8,
        true,
        true,
        true,
        true,
        false,
        false,
        'sans-serif',
        10,
        ARRAY['png', 'svg']::heatmap_format[]
    );

    -- Create generation settings with defaults
    INSERT INTO heatmap_generation_settings (user_id)
    VALUES (NEW.id);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to create default themes when user is created
CREATE TRIGGER create_default_themes_on_user_creation
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_heatmap_themes();

-- View for easily querying heatmap status per user
CREATE VIEW user_heatmap_status AS
SELECT
    u.id as user_id,
    u.username,
    COUNT(DISTINCT ht.id) as total_themes,
    COUNT(DISTINCT gh.id) as total_generated,
    COUNT(DISTINCT CASE WHEN gh.is_valid = true THEN gh.id END) as valid_generated,
    hgs.auto_generation_enabled,
    hgs.update_interval_minutes,
    hgs.next_scheduled_generation_at,
    MAX(gh.generated_at) as last_generated_at
FROM users u
LEFT JOIN heatmap_themes ht ON u.id = ht.user_id
LEFT JOIN generated_heatmaps gh ON u.id = gh.user_id
LEFT JOIN heatmap_generation_settings hgs ON u.id = hgs.user_id
GROUP BY u.id, u.username, hgs.auto_generation_enabled, hgs.update_interval_minutes, hgs.next_scheduled_generation_at;
