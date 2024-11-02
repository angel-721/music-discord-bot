use crate::types::data::Data;
use crate::types::error::Error;

#[allow(unused)]
pub type Context<'a> = poise::Context<'a, Data, Error>;
