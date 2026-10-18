use poise::CreateReply;
use serenity::{builder::CreateMessage, model::guild::Member};

use crate::{Context, Error};

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn note(
    ctx: Context<'_>,
    member: Member,
    mut note: String
) -> Result<(), Error> {
    Ok(())
}