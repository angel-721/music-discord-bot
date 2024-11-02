use crate::types::{context, error, httpkey};
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;
use songbird::input::YoutubeDl;

use rspotify::{
    model::{FullTrack, Market, TrackId},
    prelude::*,
    ClientCredsSpotify, Credentials,
};

use std::env;

async fn get_track_info(track_url: &str) -> FullTrack {
    let spotify_client_id =
        env::var("SPOTIFY_CLIENT_ID").expect("Fatality! SPOTIFY_CLIENT_ID not set!");
    let spotify_client_secret =
        env::var("SPOTIFY_CLIENT_SECRET").expect("Fatality! SPOTIFY_CLIENT_SECRET not set!");
    let creds = Credentials::new(&spotify_client_id, &spotify_client_secret);
    let spotify = ClientCredsSpotify::new(creds);
    let track_id = track_url
        .split('/')
        .last()
        .and_then(|s| s.split('?').next())
        .ok_or("Invalid URL format")
        .unwrap();

    // Create the proper Spotify URI
    let spotify_uri = format!("spotify:track:{}", track_id);
    println!("spotify_uri: {}", spotify_uri);

    // Now convert to TrackId
    let track_id = TrackId::from_uri(&spotify_uri).unwrap();
    println!("track_id: {}", track_id);

    spotify.request_token().await.unwrap();

    let track = spotify
        .track(
            track_id,
            Some(Market::Country(
                rspotify::model::Country::UnitedStates.into(),
            )),
        )
        .await
        .unwrap();
    track
}

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
    let url = String::from("https://open.spotify.com/track/1IT0WQk5J8NsaeII8ktdlZ");

    let guild_id = ctx.guild_id().unwrap();

    let track = get_track_info(&url).await;
    println!("{:?}", track);

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<httpkey::HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let search_query = format!(
        "ytsearch1:{} by {} {}",
        track.name, track.artists[0].name, "audio"
    );

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let mut src = YoutubeDl::new(http_client, search_query);
        let res = src.search(Some(1)).await.unwrap();
        println!("{:?}", res);
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
