use poise::serenity_prelude as serenity;

use crate::database::cancel_pending_unbans;
use crate::{Context, Error};

#[poise::command(
    slash_command,
    default_member_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS"
)]
pub async fn unban(
    ctx: Context<'_>,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = data.guild_id;

    // First, unban the user in Discord.
    guild_id
        .unban(&ctx.http(), user_id)
        .await?;

    let cancelled = cancel_pending_unbans(
        &data.db,
        i64::try_from(user_id.get())?,
    )
    .await?;

    ctx.say(format!(
        "Successfully unbanned <@{}> and cancelled {} pending automatic unban job(s).",
        user_id,
        cancelled
    ))
    .await?;

    Ok(())
}