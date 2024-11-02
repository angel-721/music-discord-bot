use std::sync::atomic::AtomicU32;
/// Custom user data passed to all command functions
pub struct Data {
    pub poise_mentions: AtomicU32,
}
