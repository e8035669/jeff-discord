// use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ActivityData, ActivityType};

#[allow(dead_code)]
pub fn custom_activity<N>(message: N) -> ActivityData
where
    N: ToString,
{
    let mut act = ActivityData::playing(message.to_string());
    act.kind = ActivityType::Custom;
    act
}

#[allow(dead_code)]
pub fn argsort<T: Ord>(data: &[T]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &data[i]);
    indices
}
