use chrono::Utc;
use serenity::builder::CreateMessage;
use poise::serenity_prelude as serenity;

use crate::{Context, Error, db::database, types::ModerationAction};

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn warn(
    ctx: Context<'_>,
    user_id: serenity::UserId,
    #[description = "Warn reason, anything above 1950 characters will be truncated in line wtih Discord's message limit."]
    mut reason: String,
) -> Result<(), Error> {
    let user = ctx.http().get_user(user_id).await?;
    let user_id_db = i64::try_from(user_id.get())?;
    let moderator_id = i64::try_from(ctx.author().id.get())?;

    // Truncate the string so it doesn't go over the char limit
    reason.truncate(1950);
    let dm = CreateMessage::new().content(format!("You have been warned.\nReason: {}", reason));
    if let Err(err) = user.dm(&ctx.http(), dm).await {
        eprintln!("Failed to DM user: {err:?}");
        ctx.say("Failed to notify user because of closed DMs")
            .await?;
        // Return early as to not send a confirmation message
        return Ok(());
    }
    let _case_id = match database::insert_modcase(
        &ctx.data().db,
        user_id_db,
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

    ctx.say(format!(
        "Warned <@{}> reason: {}.",
        user_id_db,
        reason
    ))
    .await?;
    Ok(())
}
