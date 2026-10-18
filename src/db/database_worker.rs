use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use serenity::{GuildId, Http, UserId};
use sqlx::PgPool;

use crate::{types::QueueRecord, types::QueuedJobType};

type WorkerError = Box<dyn std::error::Error + Send + Sync>;

pub async fn run_worker(
    db: PgPool,
    http: Arc<Http>,
    guild_id: GuildId,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));

    loop {
        interval.tick().await;

        if let Err(error) = process_due_jobs(&db, &http, guild_id).await {
            eprintln!("Error processing queued jobs: {error}");
        }
    }
}

async fn process_due_jobs(
    db: &PgPool,
    http: &Http,
    guild_id: GuildId,
) -> Result<(), WorkerError> {
    let jobs = sqlx::query_as::<_, QueueRecord>(
        r#"
        SELECT *
        FROM queued_jobs
        WHERE status = 'pending'
          AND execute_at <= NOW()
        ORDER BY execute_at ASC
        LIMIT 10
        "#,
    )
    .fetch_all(db)
    .await?;

    for job in jobs {
        let result = match job.job_type {
            QueuedJobType::Unban => {
                process_unban(http, guild_id, &job).await
            }
            QueuedJobType::Remind => {
                process_reminder(http, &job).await
            }
        };

        match result {
            Ok(()) => {
                sqlx::query(
                    r#"
                    UPDATE queued_jobs
                    SET status = 'completed'
                    WHERE id = $1 AND status = 'pending'
                    "#,
                )
                .bind(job.id)
                .execute(db)
                .await?;
            }
            Err(error) => {
                eprintln!(
                    "Failed to process job {} ({:?}): {}",
                    job.id, job.job_type, error
                );

                sqlx::query(
                    r#"
                    UPDATE queued_jobs
                    SET execute_at = NOW() + INTERVAL '1 minute'
                    WHERE id = $1 AND status = 'pending'
                    "#,
                )
                .bind(job.id)
                .execute(db)
                .await?;
            }
        }
    }

    Ok(())
}

async fn process_unban(
    http: &Http,
    guild_id: GuildId,
    job: &QueueRecord,
) -> Result<(), WorkerError> {
    let user_id = job
        .user_id
        .and_then(|id| u64::try_from(id).ok())
        .map(UserId::new)
        .ok_or("Unban job has an invalid or missing user ID")?;

    guild_id.unban(http, user_id).await?;

    println!("Automatically unbanned user {}", user_id);

    Ok(())
}

async fn process_reminder(
    http: &Http,
    job: &QueueRecord,
) -> Result<(), WorkerError> {
    let channel_id = job
        .channel_id
        .and_then(|id| u64::try_from(id).ok())
        .map(serenity::ChannelId::new)
        .ok_or("Reminder job has an invalid or missing channel ID")?;

    let user_id = job
        .user_id
        .and_then(|id| u64::try_from(id).ok())
        .map(serenity::UserId::new)
        .ok_or("Reminder job has an invalid or missing user ID")?;

    let message = job
        .payload
        .get("message")
        .and_then(serde_json::Value::as_str)
        .ok_or("Reminder job has no message in its payload")?;

    // Discord timestamp for when the reminder was created.
    let created_timestamp = job.created_at.timestamp();

    let content = format!(
        "<@{}> your reminder from <t:{}:F>\n\
         {}",
        user_id.get(),
        created_timestamp,
        message
    );

    channel_id.say(http, content).await?;

    println!("Sent reminder for job {}", job.id);

    Ok(())
}