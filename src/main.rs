mod commands;

use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

use serenity::{
    async_trait,
    client::ClientBuilder,
    // utils::MessageBuilder,
    framework::StandardFramework,
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use sqlx::postgres::PgPoolOptions;

use commands::color::*;
use commands::talking::*;
use commands::utils::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        info!("{}: {}", msg.author.name, msg.content)
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        tokio::spawn(async move {
            let color_data = _ctx
                .data
                .read()
                .await
                .get::<ColorRandomDataContainer>()
                .unwrap()
                .clone();
            color_data.update_loop(&_ctx).await;
        });
    }
}

async fn run_bot(config: HashMap<&str, &str>) {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(LevelFilter::WARN.into())
        .add_directive("jeff_discord=DEBUG".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    let http = Http::new_with_token(config["token"]);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("$"))
        .group(&TALKING_GROUP)
        .group(&COLOR_GROUP);

    let mut client = ClientBuilder::new_with_http(http)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    {
        let mut data = client.data.write().await;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(config["database_url"])
            .await
            .expect("Cannot connect to db");

        let color_data = ColorRandomData::new(pool.clone());
        color_data.init().await;

        data.insert::<PgContainer>(pool);
        data.insert::<ColorRandomDataContainer>(Arc::new(color_data));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[tokio::main]
async fn main() {
    let mut config: HashMap<&str, &str> = HashMap::new();

    let token = env::var("DC_TOKEN").expect("token not set");
    let database_url = env::var("DATABASE_URL").expect("DB url not set");

    config.insert("token", token.as_str());
    config.insert("database_url", database_url.as_str());

    run_bot(config).await;
}
