use chrono::{Duration as ChronoDuration, Utc};

use crate::{Context, Error, db::database, types::QueuedJobType};

#[poise::command(slash_command,)]
pub async fn remind(
    ctx: Context<'_>,

    #[description = "Reminder message (maximum 1900 characters)"]
    mut remind_message: String,

    #[description = "When to send the reminder, e.g. 30m, 2h, or 7d"]
    length: String,
) -> Result<(), Error> {
    let channel_id = i64::try_from(ctx.channel_id().get())?;
    // Truncate the message to stay within Discord's message limit.
    remind_message.truncate(1900);
    let user_id = i64::try_from(ctx.author().id.get())?;

    // Parse and validate the reminder duration.
    let duration = match humantime::parse_duration(&length) {
        Ok(duration) if !duration.is_zero() => duration,
        _ => {
            ctx.say(
                "Invalid duration! Try `30m`, `2h`, or `7d`. \
                 The duration must be greater than zero.",
            )
            .await?;

            return Ok(());
        }
    };

    // Calculate the reminder time safely.
    let chrono_duration = match ChronoDuration::from_std(duration) {
        Ok(duration) => duration,
        Err(_) => {
            ctx.say("That reminder duration is too large. Please use a shorter duration.")
                .await?;
            return Ok(());
        }
    };

    let execute_at = match Utc::now().checked_add_signed(chrono_duration) {
        Some(time) => time,
        None => {
            ctx.say("That reminder duration is too large. Please use a shorter duration.")
                .await?;
            return Ok(());
        }
    };

    database::enqueue_job(
        &ctx.data().db,
        QueuedJobType::Remind,
        execute_at,
        Some(user_id),
        Some(channel_id),
        serde_json::json!({
            "message": remind_message,
        }),
    )
    .await?;

    // Confirm the scheduled reminder.
    ctx.say(format!(
        "Reminder scheduled for <t:{}:F>",
        execute_at.timestamp(),
    ))
    .await?;

    Ok(())
}