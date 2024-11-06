use serenity::all::{
    model::channel::Message, CreateEmbed, CreateMessage, Result as SerenityResult,
};

// Checks that a message successfully sent; if not, then logs why to stdout.
pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

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
