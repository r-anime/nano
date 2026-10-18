use chrono::{Duration as ChronoDuration, Utc};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateMessage;

use crate::{Context, Error, db::database, types::QueuedJobType};

#[poise::command(
    slash_command,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"]
    member: serenity::Member,
    #[description = "Ban reason (maximum 512 characters)"]
    mut reason: String,
    #[description = "Ban duration, e.g. 30m, 2h, or 7d. Omit for a permanent ban."]
    length: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.data().guild_id;
    let user_id = member.user.id;

    // Truncate the reason to Discord's limit.
    reason.truncate(512);

    // Calculate the expiration time, if this is a temporary ban.
    let expires_at = if let Some(length) = length {
        let duration = match humantime::parse_duration(&length) {
            Ok(duration) if !duration.is_zero() => duration,
            _ => {
                ctx.say(
                    "Invalid duration! Try `30m`, `2h`, or `7d`. \
                    The duration must be greater than zero."
                )
                .await?;
                return Ok(());
            }
        };

        Some(Utc::now() + ChronoDuration::from_std(duration)?)
    } else { None };

    // Notify the user. This is done before the ban goes through so technically it could fail, but otherwise the DM won't reach them.
    let dm_content = if let Some(expires_at) = expires_at {
        let timestamp = expires_at.timestamp();

        format!(
            "You have been temporarily banned from the server.\n\
             Reason: {}\n\
             Expires: <t:{}:F>\n\
             Time remaining: <t:{}:R>",
            reason, timestamp, timestamp
        )
    } else {
        format!(
            "You have been permanently banned from the server.\n\
             Reason: {}",
            reason
        )
    };

    let dm = CreateMessage::new().content(dm_content);

    let dm_failed = if let Err(err) = member.user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM banned user: {err:?}");
        true
    } else {
        false
    };

    // Apply the ban
    if let Err(err) = guild_id
        .ban_with_reason(&ctx.http(), user_id, 0, &reason)
        .await
    {
        eprintln!("Failed to ban member: {err:?}");
        ctx.say(
            "Failed to ban this member. They may have a higher or equal role \
             than the bot, or the bot may be missing the Ban Members permission."
        )
        .await?;
        return Ok(());
    }

    // This is to make sure that a user doesn't accidentally get unbanned from a previous (possibly shorter) ban
    let user_id_db = i64::try_from(user_id.get())?;
    let cancelled = database::cancel_pending_unbans(
        &ctx.data().db,
        user_id_db,
    )
    .await?;
    if cancelled > 0 {
        println!(
            "Cancelled {cancelled} previously scheduled unban(s) for user {}",
            user_id
        );
    }

    // Queue the automatic unban only after the ban succeeds.
    if let Some(expires_at) = expires_at {
        if let Err(err) = database::enqueue_job(
            &ctx.data().db,
            QueuedJobType::Unban,
            expires_at,
            Some(i64::try_from(user_id.get())?),
            None,
            serde_json::json!({}),
        )
        .await
        {
            eprintln!(
                "User {} was banned, but scheduling the unban failed: {err}",
                user_id
            );

            ctx.say(format!(
                "Banned <@{}>, but **failed to schedule the automatic unban**. \
                 Please arrange the unban manually.\nReason: {}",
                user_id.get(),
                reason
            ))
            .await?;
            return Ok(());
        }
    }

    // Confirm the ban.
    let confirmation = if let Some(expires_at) = expires_at {
        let timestamp = expires_at.timestamp();

        format!(
            "Temporarily banned <@{}>.\n\
             Reason: {}\n\
             Expires: <t:{}:F> (<t:{}:R>)",
            user_id.get(),
            reason,
            timestamp,
            timestamp
        )
    } else {
        format!(
            "Permanently banned <@{}>.\nReason: {}",
            user_id.get(),
            reason
        )
    };

    let confirmation = if dm_failed {
        format!("{confirmation}\nCouldn't DM the user, they may have DMs disabled.")
    } else {
        confirmation
    };

    ctx.say(confirmation).await?;

    Ok(())
}