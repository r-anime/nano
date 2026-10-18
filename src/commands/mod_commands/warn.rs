use serenity::{builder::CreateMessage, model::guild::Member};

use crate::{Context, Error};

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn warn(
    ctx: Context<'_>,
    member: Member,
    #[description = "Warn reason, anything above 1950 characters will be truncated in line wtih Discord's message limit."]
    mut reason: String
) -> Result<(), Error> {
    // Truncate the string so it doesn't go over the char limit
    reason.truncate(1950);
    let dm = CreateMessage::new().content(format!("You have been warned.\nReason: {}",reason));
    if let Err(err) = member.user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM user: {err:?}");
        ctx.say("Failed to notify user because of closed DMs").await?;
        // Return early as to not send a confirmation message
        return Ok(())
    }
    ctx.say(format!("Warned <@{}> reason: {}.", member.user.id.get(), reason)).await?;
    Ok(())
}