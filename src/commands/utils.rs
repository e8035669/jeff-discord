use serenity::prelude::*;
use sqlx::Postgres;
use sqlx::Pool;

pub struct PgContainer;

impl TypeMapKey for PgContainer {
    type Value = Pool<Postgres>;
}

