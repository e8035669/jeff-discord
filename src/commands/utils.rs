use serenity::prelude::*;
use serenity::{model::gateway::Activity, model::gateway::ActivityType};
use sqlx::Any;
use sqlx::Pool;

pub struct PgContainer;

impl TypeMapKey for PgContainer {
    type Value = Pool<Any>;
}

#[allow(dead_code)]
pub fn custom_activity<N>(message: N) -> Activity
where
    N: ToString,
{
    let mut act = Activity::playing(message.to_string());
    act.kind = ActivityType::Custom;
    act
}

#[allow(dead_code)]
pub fn argsort<T: Ord>(data: &[T]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &data[i]);
    indices
}
