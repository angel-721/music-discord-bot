use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub type SharedQueue<T> = Arc<Mutex<VecDeque<T>>>;

#[derive(Debug, Clone)]
pub struct Song {
    pub song_name: String,
    pub artist_name: String,
    pub song_url: String,
    pub album_cover_url: String,
}

impl Song {
    pub fn new(
        song_name: String,
        artist_name: String,
        song_url: String,
        album_cover_url: String,
    ) -> Song {
        Song {
            song_name,
            artist_name,
            song_url,
            album_cover_url,
        }
    }
}

/// Custom user data passed to all command functions
pub struct Data {
    pub song_queue: SharedQueue<Song>,
}
