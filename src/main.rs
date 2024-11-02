use poise::serenity_prelude as serenity_p;
use reqwest::Client as HttpClient;
use serenity::prelude::GatewayIntents;
use songbird::SerenityInit;
use std::env;
use std::sync::atomic::{AtomicU32, Ordering};

use warped_tour_discord_bot::commands::join::join;
use warped_tour_discord_bot::commands::play::play;
use warped_tour_discord_bot::types::data::Data;
use warped_tour_discord_bot::types::error::Error;
use warped_tour_discord_bot::types::httpkey::HttpKey;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Fatality! DISCORD_TOKEN not set!");

    tracing_subscriber::fmt::init();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![join(), play()],
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
                    poise_mentions: AtomicU32::new(0),
                })
            })
        })
        .build();

    let client = serenity_p::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity_p::Context,
    event: &serenity_p::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity_p::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity_p::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("poise")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
                new_message
                    .reply(
                        ctx,
                        format!("Poise has been mentioned {} times", old_mentions + 1),
                    )
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
