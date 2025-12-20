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
