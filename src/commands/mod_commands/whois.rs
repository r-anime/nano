use serenity::{builder::CreateMessage, model::guild::Member};

use crate::{Context, Error};

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn whois(
    ctx: Context<'_>,
    member: Member,
) -> Result<(), Error> {
    // Truncate the string so it doesn't go over the char limit
    
    Ok(())
}