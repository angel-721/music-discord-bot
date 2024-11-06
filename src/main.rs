use poise::serenity_prelude as serenity;
use reqwest::Client as HttpClient;
use serenity::prelude::GatewayIntents;
use songbird::SerenityInit;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use std::env;

use warped_tour_discord_bot::commands::{join::join, play::play_playlist};
use warped_tour_discord_bot::types::{data::*, error::Error, httpkey::HttpKey};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Fatality! DISCORD_TOKEN not set!");

    tracing_subscriber::fmt::init();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![join(), play_playlist()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                Ok(Data {
                    song_queue: Arc::new(Mutex::new(VecDeque::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}
