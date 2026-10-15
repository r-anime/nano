use poise::CreateReply;
use serenity::{builder::CreateMessage, model::guild::Member};

use crate::{Context, Error};

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn kick(
    ctx: Context<'_>,
    member: Member,
    reason: String
) -> Result<(), Error> {
    let dm = CreateMessage::new().content(format!("You have been kicked from the server.\nReason: {}",reason));
    if let Err(err) = member.user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM user: {err:?}");
        ctx.say("Failed to notify user because of closed DMs").await?;
    }
    member.kick_with_reason(&ctx.http(), &reason).await?;
    ctx.say(format!("Kicked <@{}> reason: {}.", member.user.id.get(), reason)).await?;
    Ok(())
}