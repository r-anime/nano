use chrono::{Duration as ChronoDuration, Utc};
use poise::serenity_prelude as serenity;

use crate::{
    Context, Error,
    db::database,
    types::{ModerationAction, QueuedJobType},
};

#[poise::command(
    slash_command,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The Discord user ID to ban"] user_id: serenity::UserId,
    #[description = "Ban reason (maximum 512 characters)"] mut reason: String,
    #[description = "Duration, e.g. 30m, 2h, or 7d. Omit for a permanent ban."] length: Option<
        String,
    >,
) -> Result<(), Error> {
    let guild_id = ctx.data().guild_id;

    let moderator_id = i64::try_from(ctx.author().id.get())?;
    let user_id_db = i64::try_from(user_id.get())?;

    let mod_duration = length.clone().unwrap_or_else(|| "inf".to_string());

    // Truncate the reason.
    reason.truncate(512);

    // Calculate the expiration time for a temporary ban.
    let expires_at = if let Some(ref length) = length {
        let duration = match humantime::parse_duration(length) {
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
        let chrono_duration = match ChronoDuration::from_std(duration) {
            Ok(duration) => duration,
            Err(_) => {
                ctx.say("That ban duration is too large.").await?;
                return Ok(());
            }
        };
        match Utc::now().checked_add_signed(chrono_duration) {
            Some(time) => Some(time),
            None => {
                ctx.say("That ban duration is too large.").await?;
                return Ok(());
            }
        }
    } else {
        None
    };

    // Try to DM the user if their profile is accessible.
    // Failure to DM must not prevent the ban.
    let dm_failed = match ctx.http().get_user(user_id).await {
        Ok(user) => {
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
            user.dm(
                &ctx.http(),
                serenity::builder::CreateMessage::new().content(dm_content),
            )
            .await
            .is_err()
        }
        Err(err) => {
            eprintln!("Couldn't fetch user {} for DM: {err}", user_id);
            true
        }
    };

    if let Err(err) = guild_id
        .ban_with_reason(&ctx.http(), user_id, 0, &reason)
        .await
    {
        eprintln!("Failed to ban user {}: {err}", user_id);

        ctx.say(
            "Failed to ban this user. Check the ID, bot permissions, \
             and role hierarchy.",
        )
        .await?;
        return Ok(());
    }

    // Cancel previous pending automatic unbans.
    let cancelled = match database::cancel_pending_unbans(&ctx.data().db, user_id_db).await {
        Ok(count) => count,
        Err(err) => {
            eprintln!(
                "Banned {}, but failed to cancel old unban jobs: {err}",
                user_id
            );
            ctx.say(format!(
                "Banned <@{}>, but failed to cancel previous automatic \
                 unban jobs. Please check the job queue.",
                user_id.get()
            ))
            .await?;
            return Ok(());
        }
    };

    if cancelled > 0 {
        println!("Cancelled {cancelled} old unban job(s) for {}", user_id);
    }

    // Schedule the new automatic unban, if temporary.
    if let Some(expires_at) = expires_at {
        if let Err(err) = database::enqueue_job(
            &ctx.data().db,
            QueuedJobType::Unban,
            expires_at,
            Some(user_id_db),
            None,
            serde_json::json!({}),
        )
        .await
        {
            eprintln!("Banned {}, but failed to schedule unban: {err}", user_id);

            ctx.say(format!(
                "Banned <@{}>, but failed to schedule the automatic unban. \
                 Please arrange the unban manually.",
                user_id.get()
            ))
            .await?;

            return Ok(());
        }
    }

    // Record the moderation case.
    if let Err(err) = database::insert_modcase(
        &ctx.data().db,
        user_id_db,
        moderator_id,
        ModerationAction::Ban,
        reason.clone(),
        Some(mod_duration),
        Utc::now(),
    )
    .await
    {
        eprintln!(
            "Banned {}, but failed to record moderation case: {err}",
            user_id
        );
    }
    // Confirm the ban.
    let mut confirmation = if let Some(expires_at) = expires_at {
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
    if dm_failed {
        confirmation.push_str(
            "\nCouldn't DM the user; they may have DMs disabled \
             or their profile may be inaccessible.",
        );
    }
    ctx.say(confirmation).await?;

    Ok(())
}
