{%- if database != "none" %}
-- Add user preferences table
-- This migration demonstrates how to add new functionality to an existing schema

{%- if database == "postgres" %}
-- Create user preferences table
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key VARCHAR(100) NOT NULL,
    preference_value TEXT,
    preference_type VARCHAR(50) DEFAULT 'string' CHECK (preference_type IN ('string', 'number', 'boolean', 'json')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, preference_key)
);

-- Create indexes
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

-- Create trigger for automatic updated_at
CREATE TRIGGER update_user_preferences_updated_at 
    BEFORE UPDATE ON user_preferences 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Add some default preferences for existing users
INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type)
SELECT 
    id,
    'theme',
    'light',
    'string'
FROM users
WHERE is_active = true;

INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type)
SELECT 
    id,
    'notifications_enabled',
    'true',
    'boolean'
FROM users
WHERE is_active = true;

INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type)
SELECT 
    id,
    'items_per_page',
    '20',
    'number'
FROM users
WHERE is_active = true;

{%- elif database == "sqlite" %}
-- Create user preferences table for SQLite
CREATE TABLE user_preferences (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key TEXT NOT NULL,
    preference_value TEXT,
    preference_type TEXT DEFAULT 'string' CHECK (preference_type IN ('string', 'number', 'boolean', 'json')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, preference_key)
);

-- Create indexes
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

-- Create trigger for automatic updated_at
CREATE TRIGGER update_user_preferences_updated_at 
    AFTER UPDATE ON user_preferences
    FOR EACH ROW
BEGIN
    UPDATE user_preferences SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Add some default preferences for existing users
INSERT INTO user_preferences (id, user_id, preference_key, preference_value, preference_type)
SELECT 
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || 
    substr(lower(hex(randomblob(2))),2) || '-' || 
    substr('89ab',abs(random()) % 4 + 1, 1) || 
    substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6))),
    id,
    'theme',
    'light',
    'string'
FROM users
WHERE is_active = 1;

INSERT INTO user_preferences (id, user_id, preference_key, preference_value, preference_type)
SELECT 
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || 
    substr(lower(hex(randomblob(2))),2) || '-' || 
    substr('89ab',abs(random()) % 4 + 1, 1) || 
    substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6))),
    id,
    'notifications_enabled',
    'true',
    'boolean'
FROM users
WHERE is_active = 1;

{%- elif database == "mysql" %}
-- Create user preferences table for MySQL
CREATE TABLE user_preferences (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    preference_key VARCHAR(100) NOT NULL,
    preference_value TEXT,
    preference_type ENUM('string', 'number', 'boolean', 'json') DEFAULT 'string',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY unique_user_preference (user_id, preference_key),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

-- Add some default preferences for existing users
INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type)
SELECT 
    id,
    'theme',
    'light',
    'string'
FROM users
WHERE is_active = TRUE;

INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type)
SELECT 
    id,
    'notifications_enabled',
    'true',
    'boolean'
FROM users
WHERE is_active = TRUE;
{%- endif %}

-- Create a view for easy access to user preferences
{%- if database == "postgres" %}
CREATE VIEW user_preferences_view AS
SELECT 
    u.id as user_id,
    u.username,
    u.email,
    json_object_agg(up.preference_key, up.preference_value) as preferences
FROM users u
LEFT JOIN user_preferences up ON u.id = up.user_id
WHERE u.is_active = true
GROUP BY u.id, u.username, u.email;

{%- elif database == "sqlite" %}
-- SQLite doesn't have json_object_agg, so we'll create a simpler view
CREATE VIEW user_preferences_view AS
SELECT 
    u.id as user_id,
    u.username,
    u.email,
    up.preference_key,
    up.preference_value,
    up.preference_type
FROM users u
LEFT JOIN user_preferences up ON u.id = up.user_id
WHERE u.is_active = 1;

{%- elif database == "mysql" %}
CREATE VIEW user_preferences_view AS
SELECT 
    u.id as user_id,
    u.username,
    u.email,
    JSON_OBJECTAGG(up.preference_key, up.preference_value) as preferences
FROM users u
LEFT JOIN user_preferences up ON u.id = up.user_id
WHERE u.is_active = TRUE
GROUP BY u.id, u.username, u.email;
{%- endif %}

-- Add some sample preferences data for demonstration
{%- if database == "postgres" %}
INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type) VALUES
    ('00000000-0000-0000-0000-000000000001', 'sidebar_collapsed', 'false', 'boolean'),
    ('00000000-0000-0000-0000-000000000001', 'language', 'en', 'string'),
    ('00000000-0000-0000-0000-000000000001', 'timezone', 'UTC', 'string'),
    ('00000000-0000-0000-0000-000000000002', 'theme', 'dark', 'string'),
    ('00000000-0000-0000-0000-000000000002', 'email_frequency', 'weekly', 'string'),
    ('00000000-0000-0000-0000-000000000003', 'auto_save', 'true', 'boolean'),
    ('00000000-0000-0000-0000-000000000003', 'items_per_page', '50', 'number');

{%- elif database == "sqlite" %}
INSERT INTO user_preferences (id, user_id, preference_key, preference_value, preference_type) VALUES
    ('50000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'sidebar_collapsed', 'false', 'boolean'),
    ('50000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'language', 'en', 'string'),
    ('50000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000002', 'theme', 'dark', 'string'),
    ('50000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000003', 'auto_save', 'true', 'boolean');

{%- elif database == "mysql" %}
INSERT INTO user_preferences (user_id, preference_key, preference_value, preference_type) VALUES
    ('00000000-0000-0000-0000-000000000001', 'sidebar_collapsed', 'false', 'boolean'),
    ('00000000-0000-0000-0000-000000000001', 'language', 'en', 'string'),
    ('00000000-0000-0000-0000-000000000002', 'theme', 'dark', 'string'),
    ('00000000-0000-0000-0000-000000000003', 'auto_save', 'true', 'boolean');
{%- endif %}
{%- endif %}