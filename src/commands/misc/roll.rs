use rand::RngExt;

use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Min (inclusive)"] n1: i64,
    #[description = "Max (inclusive)"] n2: i64,
) -> Result<(), Error> {
    let min = n1.min(n2);
    let max = n1.max(n2);

    let result = rand::rng().random_range(min..=max);

    ctx.say(format!("You rolled **{}**!", result)).await?;

    Ok(())
}
