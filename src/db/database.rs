use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::types::{QueueRecord, QueuedJobStatus, QueuedJobType};

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