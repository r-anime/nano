use rand::RngExt;

use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Min (inclusive)"] min: i64,
    #[description = "Max (inclusive)"] max: i64,
) -> Result<(), Error> {
    let actual_min = min.min(max);
    let actual_max = min.max(max);

    let result = rand::rng().random_range(actual_min..=actual_max);

    ctx.say(format!("You rolled **{}**!", result)).await?;

    Ok(())
}
