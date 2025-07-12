-- Create blacklist_entries table
CREATE TABLE blacklist_entries (
    id BIGSERIAL PRIMARY KEY,
    channel_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    custom_message TEXT,
    UNIQUE(channel_id, role_id)
);

-- Create exception_entries table
CREATE TABLE exception_entries (
    id BIGSERIAL PRIMARY KEY,
    channel_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    UNIQUE(channel_id, role_id)
);

-- Create indexes for efficient queries
CREATE INDEX idx_blacklist_channel_id ON blacklist_entries(channel_id);
CREATE INDEX idx_blacklist_role_id ON blacklist_entries(role_id);
CREATE INDEX idx_blacklist_channel_role ON blacklist_entries(channel_id, role_id);

CREATE INDEX idx_exception_channel_id ON exception_entries(channel_id);
CREATE INDEX idx_exception_role_id ON exception_entries(role_id);
CREATE INDEX idx_exception_channel_role ON exception_entries(channel_id, role_id);
