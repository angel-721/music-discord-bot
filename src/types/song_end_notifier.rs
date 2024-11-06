use crate::types::data::Song;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};
use std::{collections::VecDeque, sync::Arc};

use serenity::all::{async_trait, http::Http, model::prelude::ChannelId};
use tokio::sync::Mutex;

use crate::messages::messages::playing_song_message;

pub struct SongEndNotifier {
    pub chan_id: ChannelId,
    pub http: Arc<Http>,
    pub queue: Arc<Mutex<VecDeque<Song>>>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        if self.queue.lock().await.len() <= 0 {
            println!("Done with playlist");
            let _ = self
                .chan_id
                .say(self.http.clone(), "Done playing playlist")
                .await
                .unwrap();
            return None;
        }
        let mut queue = self.queue.lock().await;
        let song = queue.pop_front().unwrap();
        let msg_embed = playing_song_message(
            song.artist_name.clone(),
            song.song_name.clone(),
            song.album_cover_url.clone(),
            song.song_url.clone(),
        )
        .await;

        self.chan_id
            .send_message(self.http.clone(), msg_embed)
            .await
            .unwrap();
        None
    }
}
