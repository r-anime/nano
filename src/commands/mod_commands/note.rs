use chrono::Utc;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::{Context, Error, db::database, types::ModerationAction};

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn note(
    ctx: Context<'_>,
    user_id: serenity::UserId,
    mut note: String,
) -> Result<(), Error> {
    // Limit the note length.
    note.truncate(1900);

    let user_id_db = i64::try_from(user_id.get())?;
    let moderator_id = i64::try_from(ctx.author().id.get())?;

    let _case_id = match database::insert_modcase(
        &ctx.data().db,
        user_id_db,
        moderator_id,
        ModerationAction::Note,
        note.clone(),
        None,
        Utc::now(),
    )
    .await
    {
        Ok(id) => id,
        Err(err) => {
            eprintln!("Failed to save moderation note: {err}");
            ctx.send(
                CreateReply::default()
                    .content("Failed to save the note. Please try again.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    ctx.send(CreateReply::default().content(format!("Saved note for <@{}>: {}", user_id_db, note)))
        .await?;

    Ok(())
}
