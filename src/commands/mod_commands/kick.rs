use poise::serenity_prelude as serenity;
use serenity::builder::CreateMessage;

use crate::{Context, Error};

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

    ctx.say(format!(
        "Kicked <@{}>. Reason: {}.{}",
        member.user.id.get(),
        reason,
        notification
    ))
    .await?;

    Ok(())
}