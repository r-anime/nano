use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::types::{self, QueueRecord, QueuedJobStatus, QueuedJobType};

pub async fn enqueue_job(
    db: &PgPool,
    job_type: QueuedJobType,
    execute_at: DateTime<Utc>,
    user_id: Option<i64>,
    channel_id: Option<i64>,
    payload: Value,
) -> Result<i64, sqlx::Error> {
    let job = sqlx::query_as::<_, QueueRecord>(
        r#"
        INSERT INTO queued_jobs (
            job_type,
            execute_at,
            user_id,
            channel_id,
            payload
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(job_type)
    .bind(execute_at)
    .bind(user_id)
    .bind(channel_id)
    .bind(payload)
    .fetch_one(db)
    .await?;

    Ok(job.id)
}


pub async fn cancel_pending_unbans(
    db: &PgPool,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE queued_jobs
        SET status = $1
        WHERE job_type = $2
          AND status = $3
          AND user_id = $4
        "#,
    )
    .bind(QueuedJobStatus::Cancelled)
    .bind(QueuedJobType::Unban)
    .bind(QueuedJobStatus::Pending)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}


pub async fn insert_modcase(
    db: &PgPool,
    user_id: i64,
    moderator_id: i64,
    mod_action: types::ModerationAction,
    reason: String,
    duration: Option<String>,
    created_at: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO moderation_cases (
            user_id,
            moderator_id,
            action,
            reason,
            duration,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(moderator_id)
    .bind(mod_action)
    .bind(reason)
    .bind(duration)
    .bind(created_at)
    .fetch_one(db)
    .await?;

    Ok(id)
}