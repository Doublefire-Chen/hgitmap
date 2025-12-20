-- Add admin support to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN DEFAULT false;

-- OAuth applications configuration table
-- Stores OAuth app credentials configured by admins
CREATE TABLE IF NOT EXISTS oauth_applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    platform git_platform NOT NULL,
    instance_url VARCHAR(512) NOT NULL DEFAULT '', -- Empty string for official instances (github.com, gitlab.com)
    instance_name VARCHAR(255) NOT NULL, -- User-friendly name like "GitHub" or "Company GitLab"
    client_id VARCHAR(512) NOT NULL,
    client_secret TEXT NOT NULL, -- Encrypted
    is_enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false, -- Default OAuth app for this platform
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(platform, instance_url)
);

-- Trigger for updated_at
CREATE TRIGGER update_oauth_applications_updated_at BEFORE UPDATE ON oauth_applications
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Index for quick lookups
CREATE INDEX idx_oauth_apps_platform ON oauth_applications(platform, is_enabled);

-- Comments for documentation
COMMENT ON TABLE oauth_applications IS 'Stores OAuth application credentials configured by administrators';
COMMENT ON COLUMN oauth_applications.instance_url IS 'Empty string for official instances (github.com), URL for self-hosted';
COMMENT ON COLUMN oauth_applications.client_secret IS 'Encrypted OAuth client secret';
COMMENT ON COLUMN oauth_applications.is_default IS 'Default OAuth app for this platform/instance combination';
