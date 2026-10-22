use std::env;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

mod utils;
mod commands;
mod types;
mod db;

struct Data {
    db: sqlx::PgPool,
    guild_id: serenity::GuildId,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Loading all the various ENV variables
    dotenv().ok();

    let token = env::var("TOKEN")
        .expect("Missing TOKEN env var, see README for more information.");

    let host = std::env::var("DB_HOST").expect("Missing DB_HOST environment variable");
    let port = std::env::var("DB_PORT").expect("Missing DB_PORT environment variable");
    let username = std::env::var("DB_USERNAME").expect("Missing DB_USERNAME environment variable");
    let password = std::env::var("DB_PASSWORD").expect("Missing DB_PASSWORD environment variable");
    let database = std::env::var("DB_DATABASE").expect("Missing DB_DATABASE environment variable");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&format!(
            "postgres://{username}:{password}@{host}:{port}/{database}"
        ))
        .await?;

    let guild_id = env::var("GUILD_ID")
        .expect("Missing GUILD_ID environment variable")
        .parse::<u64>()
        .expect("GUILD_ID must be a valid Discord guild ID");

    let guild_id = serenity::GuildId::new(guild_id);

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;


    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::mod_commands::kick::kick(),
                commands::mod_commands::ban::ban(),
                commands::mod_commands::unban::unban(),
                commands::mod_commands::warn::warn(),
                commands::misc::remind::remind(),
                commands::mod_commands::note::note(),
                commands::mod_commands::whois::whois(),
                commands::misc::roll::roll(),
                commands::misc::pick::pick(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands,
                )
                .await?;
                    
                let data = Data {
                    db: pool,
                    guild_id,
                };

                tokio::spawn(crate::db::database_worker::run_worker(
                    data.db.clone(),
                    ctx.http.clone(),
                    data.guild_id,
                ));

                Ok(data)
            })
        })
    .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}