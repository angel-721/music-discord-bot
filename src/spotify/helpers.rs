use crate::types::data::Song;

use rspotify::{
    model::{FullTrack, Market, PlayableItem, PlaylistId, TrackId},
    prelude::*,
    ClientCredsSpotify, Credentials,
};

use std::env;

fn get_spotify_id(url: &str) -> &str {
    url.split('/')
        .last()
        .and_then(|s| s.split('?').next())
        .ok_or("Invalid URL format")
        .unwrap()
}

fn spotify_client() -> ClientCredsSpotify {
    let spotify_client_id =
        env::var("SPOTIFY_CLIENT_ID").expect("Fatality! SPOTIFY_CLIENT_ID not set!");
    let spotify_client_secret =
        env::var("SPOTIFY_CLIENT_SECRET").expect("Fatality! SPOTIFY_CLIENT_SECRET not set!");
    ClientCredsSpotify::new(Credentials::new(&spotify_client_id, &spotify_client_secret))
}

pub async fn get_playlist_songs(playlist_url: &str) -> Vec<Song> {
    let spotify = spotify_client();
    let playlist_id = get_spotify_id(playlist_url);

    let spotify_uri = format!("spotify:playlist:{}", playlist_id);
    let playlist_id = PlaylistId::from_uri(&spotify_uri).unwrap();

    spotify.request_token().await.unwrap();

    let playlist = spotify
        .playlist(
            playlist_id,
            None,
            Some(Market::Country(
                rspotify::model::Country::UnitedStates.into(),
            )),
        )
        .await
        .unwrap();

    let songs: Vec<Song> = playlist
        .tracks
        .items
        .iter()
        .filter_map(|playlist_item| playlist_item.track.clone())
        .filter_map(|playable_item| match playable_item {
            PlayableItem::Track(track) => Some(track),
            PlayableItem::Episode(_) => None,
        })
        .filter_map(|track| {
            Some(Song::new(
                track.name.clone(),
                track.artists[0].name.clone(),
                track.external_urls["spotify"].clone(),
                track.album.images[0].url.clone(),
            ))
        })
        .collect();

    songs
}

pub async fn get_track_info(track_url: &str) -> FullTrack {
    let spotify = spotify_client();
    let track_id = get_spotify_id(track_url);

    let spotify_uri = format!("spotify:track:{}", track_id);
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
