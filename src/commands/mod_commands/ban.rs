use serenity::{builder::CreateMessage, model::guild::Member};
use chrono::{Duration as ChronoDuration, Utc};

use crate::{Context, Error, database, types::QueuedJobType};

#[poise::command(
    slash_command,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    member: Member,
    #[description = "Ban reason, anything above 512 characters will be truncated in line wtih Discord's max ban reason character limit."]
    mut reason: String,
    #[description = "The lenght of the ban, if you do not include this the ban will be permant."]
    length: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.data().guild_id;
    let user_id = member.user.id;

    // Truncate the string so it doesn't go over the char limit
    reason.truncate(512);

    // Determine whether the ban is temporary or permanent.
    let (dm, confirmation) = if let Some(length) = length {
        // Parse the duration.
        let duration = match humantime::parse_duration(&length) {
            Ok(duration) if !duration.is_zero() => duration,
            _ => {
                ctx.say("Invalid duration! Try `30m`, `2h`, or `7d`.")
                    .await?;
                return Ok(());
            }
        };

        // Calculate the expiration timestamp.
        let chrono_duration = ChronoDuration::from_std(duration)?;
        let expires_at = Utc::now() + chrono_duration;
        let timestamp = expires_at.timestamp();

        let dm = CreateMessage::new().content(format!(
            "You have been temporarily banned from the server.\n\
             Reason: {}\n\
             Expires: <t:{}:F>\n\
             Time remaining: <t:{}:R>",
            reason, timestamp, timestamp
        ));

        let confirmation = format!(
            "Banned <@{}> temporarily.\n\
             Reason: {}\n\
             Expires: <t:{}:F> (<t:{}:R>)",
            user_id.get(),
            reason,
            timestamp,
            timestamp
        );

        database::enqueue_job(
            &ctx.data().db,
            QueuedJobType::Unban,
            expires_at,
            Some(user_id.get() as i64),
            None,
            serde_json::json!({}),
        )
        .await?;

        (dm, confirmation)
    } else {
        let dm = CreateMessage::new().content(format!(
            "You have been permanently banned from the server.\n\
             Reason: {}",
            reason
        ));

        let confirmation = format!(
            "Permanently banned <@{}>.\nReason: {}",
            user_id.get(),
            reason
        );

        (dm, confirmation)
    };

    // Notify the user before banning them.
    if let Err(err) = member.user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM user: {err:?}");
    }

    // Apply the ban.
    guild_id
        .ban_with_reason(&ctx.http(), user_id, 0, &reason)
        .await?;

    // Confirm the ban.
    ctx.say(confirmation).await?;

    Ok(())
}