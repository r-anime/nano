use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Type, prelude::FromRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "queued_job_type", rename_all = "lowercase")]
pub enum QueuedJobType {
    Unban,
    Remind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "queued_job_status", rename_all = "lowercase")]
pub enum QueuedJobStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, FromRow)]
pub struct QueueRecord {
    pub id: i64,
    pub job_type: QueuedJobType,
    pub execute_at: DateTime<Utc>,
    pub status: QueuedJobStatus,
    pub channel_id: Option<i64>,
    pub user_id: Option<i64>,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}