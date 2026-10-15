use chrono::{DateTime, Utc};
use serde_json::Value;

// Enum to indicate the various types of queued operations the bot an perform
pub enum QueuedJobType {
    Unban,
    Remind
}

pub enum QueuedJobStatus {
    pending,
    completed,
    failed,
    cancelled
}

pub struct QueueRecord {
    id: u64,
    job_type: QueuedJobType,
    execute_at: DateTime<Utc>,
    status: QueuedJobStatus,
    channel_id: Option<u64>,
    user_id: Option<u64>,
    payload: Value,
    creatd_at: DateTime<Utc>
}