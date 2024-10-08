use super::color::ColorRandom;
use anyhow::Error;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
pub struct Data {
    pub pool: DatabaseConnection,
    pub color_data: Arc<ColorRandom>,
}
