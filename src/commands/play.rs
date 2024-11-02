use crate::types::{context, error, httpkey};
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;
use songbird::input::YoutubeDl;

// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: context::Context<'_>,
    #[description = "Message to react to (enter a link or ID)"] msg: Message,
) -> Result<(), error::Error> {
    let url = String::from("https://youtu.be/EpWGhCTw_l8?si=RsOhbpRhJLus6dYg");

    let guild_id = ctx.guild_id().unwrap();

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<httpkey::HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let src = YoutubeDl::new(http_client, url);
        handler.play(src.into());

        check_msg(msg.channel_id.say(ctx.http(), "Playing song").await);
    } else {
        check_msg(
            msg.channel_id
                .say(ctx.http(), "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}
