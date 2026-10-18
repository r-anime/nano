-- Add migration script here
CREATE TYPE moderation_action AS ENUM (
    'ban',
    'kick',
    'warn'
);

CREATE TABLE moderation_cases (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id BIGINT NOT NULL,
    moderator_id BIGINT NOT NULL,
    action moderation_action NOT NULL,
    reason TEXT NOT NULL,
    duration TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_moderation_cases_user_id
    ON moderation_cases (user_id);