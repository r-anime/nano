use chrono::Utc;
use serenity::{builder::CreateMessage, model::guild::Member};

use crate::{Context, Error, db::database, types::ModerationAction};

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn warn(
    ctx: Context<'_>,
    member: Member,
    #[description = "Warn reason, anything above 1950 characters will be truncated in line wtih Discord's message limit."]
    mut reason: String
) -> Result<(), Error> {
    let user_id = i64::try_from(member.user.id.get())?;
    let moderator_id = i64::try_from(ctx.author().id.get())?;

    // Truncate the string so it doesn't go over the char limit
    reason.truncate(1950);
    let dm = CreateMessage::new().content(format!("You have been warned.\nReason: {}",reason));
    if let Err(err) = member.user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM user: {err:?}");
        ctx.say("Failed to notify user because of closed DMs").await?;
        // Return early as to not send a confirmation message
        return Ok(())
    }
    let _case_id = match database::insert_modcase(
        &ctx.data().db,
        user_id,
        moderator_id,
        ModerationAction::Warn,
        reason.clone(),
        None,
        Utc::now(),
    )
    .await
    {
        Ok(id) => Some(id),
        Err(err) => {
            eprintln!(
                "Warned user {}, but failed to record moderation case: {err}",
                user_id
            );
            None
        }
    };
    
    ctx.say(format!("Warned <@{}> reason: {}.", member.user.id.get(), reason)).await?;
    Ok(())
}