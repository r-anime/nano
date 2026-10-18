use chrono::Utc;
use poise::serenity_prelude as serenity;
use serenity::builder::CreateMessage;

use crate::{Context, Error, db::database, types::ModerationAction};

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The member to kick"]
    member: serenity::Member,
    #[description = "Reason for the kick (maximum 512 characters)"]
    mut reason: String,
) -> Result<(), Error> {
    let user_id = i64::try_from(member.user.id.get())?;
    let moderator_id = i64::try_from(ctx.author().id.get())?;
    
    // Truncate the reason to Discord's limit.
    reason.truncate(512);

    if let Err(err) = member
        .kick_with_reason(&ctx.http(), &reason)
        .await
    {
        eprintln!("Failed to kick member: {err:?}");
        ctx.say(
            "Failed to kick this member. They may have a higher or equal \
             role than the bot, or the bot may be missing the Kick Members permission."
        )
        .await?;
        return Ok(());
    }

    // The kick succeeded, so try to notify the user.
    let dm = CreateMessage::new().content(format!(
        "You have been kicked from the server.\nReason: {}",
        reason
    ));

    let notification = match member.user.dm(&ctx.http(), dm).await {
        Ok(_) => " I've sent them a DM.",
        Err(err) => {
            eprintln!("Failed to DM kicked user: {err:?}");
            " I couldn't send them a DM; they may have DMs disabled."
        }
    };

    // Record the kick in moderation history.
    let _case_id = match database::insert_modcase(
        &ctx.data().db,
        user_id,
        moderator_id,
        ModerationAction::Kick,
        reason.clone(),
        None,
        Utc::now(),
    )
    .await
    {
        Ok(id) => Some(id),
        Err(err) => {
            eprintln!(
                "Kicked user {}, but failed to record moderation case: {err}",
                user_id
            );
            None
        }
    };

    ctx.say(format!(
        "Kicked <@{}>. Reason: {}.{}",
        member.user.id.get(),
        reason,
        notification
    ))
    .await?;

    Ok(())
}