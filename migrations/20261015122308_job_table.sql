-- Add migration script here
CREATE TYPE queued_job_type AS ENUM (
    'unban',
    'remind'
);

CREATE TYPE queued_job_status AS ENUM (
    'pending',
    'completed',
    'failed',
    'cancelled'
);

CREATE TABLE queued_jobs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    job_type queued_job_type NOT NULL,
    execute_at TIMESTAMPTZ NOT NULL,
    status queued_job_status NOT NULL DEFAULT 'pending',
    channel_id BIGINT,
    user_id BIGINT,
    payload JSON NOT NULL DEFAULT '{}'::JSON,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_queued_jobs_pending_execute_at
    ON queued_jobs (execute_at)
    WHERE status = 'pending';

CREATE INDEX idx_queued_jobs_user_id
    ON queued_jobs (user_id);