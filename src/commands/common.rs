use super::color::ColorRandomData;
use sqlx::AnyPool;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
pub struct Data {
    pub pool: AnyPool,
    pub color_data: Arc<ColorRandomData>,
}
