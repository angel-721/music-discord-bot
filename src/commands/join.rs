use crate::types::{context, error};

#[poise::command(slash_command, prefix_command)]
pub async fn join(ctx: context::Context<'_>) -> Result<(), error::Error> {
    ctx.defer().await.unwrap();
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            let _ = ctx.reply("Not in a voice channel").await;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(_handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        // println!("Success!");
    }

    ctx.reply(format!(
        "Joined channel: {}",
        channel_id.unwrap().name(&ctx).await.unwrap()
    ))
    .await
    .unwrap();

    Ok(())
}
