use crate::types::{context, error, httpkey};
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;
use songbird::input::YoutubeDl;

use serenity::all::{CreateEmbed, CreateMessage};

use rspotify::{
    model::{FullTrack, Market, TrackId},
    prelude::*,
    ClientCredsSpotify, Credentials,
};

use std::env;

pub async fn playing_song_message(
    artist_name: String,
    song_name: String,
    cover_uri: String,
    song_uri: String,
) -> CreateMessage {
    let embed = CreateEmbed::new()
        .title("🔊 Now playing:")
        .description(format!(
            "### [{} - {}]({})",
            artist_name, song_name, song_uri
        ))
        .thumbnail(cover_uri);
    let new_m = CreateMessage::new().add_embed(embed);
    new_m
}

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

    // Now convert to TrackId
    let track_id = TrackId::from_uri(&spotify_uri).unwrap();

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
pub async fn play(ctx: context::Context<'_>) -> Result<(), error::Error> {
    let url = String::from("https://open.spotify.com/track/1IT0WQk5J8NsaeII8ktdlZ");

    let guild_id = ctx.guild_id().unwrap();

    let track = get_track_info(&url).await;

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
        let _ = src.search(Some(1)).await.unwrap();
        handler.play(src.into());

        let msg_embed = playing_song_message(
            track.artists[0].name.clone(),
            track.name,
            track.album.images[0].clone().url,
            track.external_urls["spotify"].clone(),
        )
        .await;

        ctx.channel_id()
            .send_message(ctx.http(), msg_embed)
            .await
            .unwrap();
    } else {
        check_msg(
            ctx.channel_id()
                .say(ctx.http(), "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}
