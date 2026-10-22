use crate::{Context, Error};
use rand::seq::IndexedRandom;

#[poise::command(slash_command)]
pub async fn pick(
    ctx: Context<'_>,
    #[description = "Comma-separated list of options you want to pick from"] pick_list: String,
) -> Result<(), Error> {
    let options: Vec<&str> = pick_list
        .split(',')
        .map(str::trim)
        .filter(|option| !option.is_empty())
        .collect();

    if options.is_empty() {
        ctx.say("Please provide at least one option.").await?;
        return Ok(());
    }

    let choice = options
        .choose(&mut rand::rng())
        .expect("options is not empty");

    ctx.say(format!("I pick **{}**", choice)).await?;

    Ok(())
}
