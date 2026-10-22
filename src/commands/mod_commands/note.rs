use chrono::Utc;
use poise::CreateReply;
use serenity::model::guild::Member;

use crate::{Context, Error, db::database, types::ModerationAction};

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn note(ctx: Context<'_>, member: Member, mut note: String) -> Result<(), Error> {
    // Limit the note length.
    note.truncate(1900);

    let user_id = i64::try_from(member.user.id.get())?;
    let moderator_id = i64::try_from(ctx.author().id.get())?;

    let _case_id = match database::insert_modcase(
        &ctx.data().db,
        user_id,
        moderator_id,
        ModerationAction::Note,
        note,
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

    ctx.send(CreateReply::default().content(format!("Saved note for <@{}>", member.user.id)))
        .await?;

    Ok(())
}
