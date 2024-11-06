use crate::messages::messages::*;
use crate::spotify::helpers::*;
use crate::types::{context, data::Song, error, httpkey, song_end_notifier::SongEndNotifier};
use futures::future::join_all;
use songbird::input::YoutubeDl;
use songbird::{Event, TrackEvent};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[poise::command(slash_command, prefix_command)]
pub async fn play_playlist(
    ctx: context::Context<'_>,
    #[description = "Spotify playlist link"] playlist_url: String,
) -> Result<(), error::Error> {
    ctx.defer().await.unwrap();

    let guild_id = ctx.guild_id().unwrap();

    let songs = get_playlist_songs(&playlist_url).await;

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

    let handler_lock = manager.get(guild_id).unwrap();
    let song_queue: Arc<Mutex<VecDeque<Song>>> = Arc::new(Mutex::new(VecDeque::new()));
    let mut handles = vec![]; // Collect task handles

    songs.iter().for_each(|song| {
        let search_query = format!(
            "ytsearch1:{} by {} {}",
            song.song_name, song.artist_name, "audio"
        );

        let handler_lock = handler_lock.clone();
        let song_queue = song_queue.clone();
        let http_client = http_client.clone();

        let song = song.clone(); // Clone song for move into async block

        // Spawn task and store handle
        let handle = tokio::spawn(async move {
            let mut src = YoutubeDl::new(http_client, search_query);
            match src.search(Some(1)).await {
                Ok(_) => {
                    let mut call = handler_lock.lock().await;
                    let mut dequeue = song_queue.lock().await;
                    dequeue.push_back(song);
                    call.enqueue(src.into()).await;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        });
        handles.push(handle); // Push handle to vector
    });

    // Wait for all tasks to complete
    join_all(handles).await;

    println!("Songs: {:?}", songs);
    // Now access the queue, knowing it's populated
    let mut handler = handler_lock.lock().await;
    let mut queue = song_queue.lock().await;

    handler.add_global_event(
        Event::Track(TrackEvent::End),
        SongEndNotifier {
            chan_id: ctx.channel_id(),
            queue: song_queue.clone(),
            http: ctx.serenity_context().http.clone(),
        },
    );

    let _ = handler.queue().resume();
    let song = queue.pop_front().unwrap();

    let msg_embed = playing_song_message(
        song.artist_name.clone(),
        song.song_name.clone(),
        song.album_cover_url.clone(),
        song.song_url.clone(),
    )
    .await;

    ctx.channel_id()
        .send_message(ctx.http(), msg_embed)
        .await
        .unwrap();

    //TODO: Maybe add details about the playlist?
    ctx.reply("Starting playlist").await.unwrap();

    Ok(())
}
