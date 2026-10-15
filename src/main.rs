use std::env;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;

mod utils;
mod commands;
mod types;

struct Data {
    db: sqlx::PgPool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let token = env::var("TOKEN")
        .expect("Missing TOKEN env var, see README for more information.");

    let database_url = std::env::var("DATABASE_URL")
        .expect("Missing DATABASE_URL environment variable");

    let pool = sqlx::PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;


    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::mod_commands::kick::kick(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(
                    ctx,
                    &framework.options().commands,
                )
                .await?;

                Ok(Data { db: pool })
            })
        })
    .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}