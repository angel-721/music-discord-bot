use crate::types::{context, data::Song, error, httpkey};
use songbird::{
    input::YoutubeDl, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};
use std::sync::Arc;

use serenity::all::{
    async_trait,
    http::Http,
    model::{channel::Message, prelude::ChannelId},
    CreateEmbed, CreateMessage, Result as SerenityResult,
};

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

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // TODO: Set up queue to add the song to and play the the song off the queue
    let new_song = Song::new(
        track.name.clone(),
        track.artists[0].name.clone(),
        track.external_urls["spotify"].clone(),
        track.album.images[0].url.clone(),
    );

    // let mut queue = ctx.data().song_queue.lock().unwrap();

    ctx.data().song_queue.lock().unwrap().push_back(new_song);

    let song_to_play = ctx.data().song_queue.lock().unwrap().pop_front().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        ctx.defer().await.unwrap();
        let mut handler = handler_lock.lock().await;

        let search_query = format!(
            "ytsearch1:{} by {} {}",
            song_to_play.song_name, song_to_play.artist_name, "audio"
        );

        let mut src = YoutubeDl::new(http_client, search_query);
        let _ = src.search(Some(1)).await.unwrap();
        let song = handler.play(src.into());

        let _ = song.add_event(
            Event::Track(TrackEvent::End),
            SongEndNotifier {
                chan_id: ctx.channel_id(),
                http: ctx.serenity_context().http.clone(),
            },
        );

        let msg_embed = playing_song_message(
            song_to_play.artist_name,
            song_to_play.song_name,
            song_to_play.album_cover_url,
            song_to_play.song_url,
        )
        .await;

        ctx.reply("hi").await.unwrap();
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

struct SongEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        check_msg(
            self.chan_id
                .say(&self.http, "Song faded out completely!")
                .await,
        );

        None
    }
}
