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
    framework::standard::help_commands,
    framework::standard::macros::help,
    framework::standard::macros::hook,
    framework::standard::Args,
    framework::standard::CommandError,
    framework::standard::CommandGroup,
    framework::standard::CommandResult,
    framework::standard::HelpOptions,
    framework::StandardFramework,
    http::Http,
    model::channel::Message,
    model::gateway::Activity,
    model::gateway::Ready,
    model::id::UserId,
    model::prelude::*,
    prelude::*,
    //
};

use sqlx::any::AnyPoolOptions;

use commands::color::*;
use commands::statistic::*;
use commands::talking::*;
use commands::utils::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        info!("{}: {}", msg.author.name, msg.content);
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        _ctx.set_activity(Activity::listening("來自內心的聲音"))
            .await;

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

#[hook]
async fn before_hook(_ctx: &Context, _msg: &Message, _command_name: &str) -> bool {
    println!("Running command {}", _command_name);
    true
}

#[hook]
async fn after_hook(
    _ctx: &Context,
    _msg: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>,
) {
    if let Err(why) = error {
        println!("Error in {}: {:?}", cmd_name, why);
    }
}

#[help]
#[individual_command_tip = "嗨 我是薏仁的機器人，可以用的指令如下："]
#[command_not_found_text = "找不到指令: '{}'"]
#[strikethrough_commands_tip_in_guild = ""]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

async fn run_bot(config: HashMap<&str, &str>) {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(LevelFilter::WARN.into())
        .add_directive("jeff_discord=DEBUG".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    let http = Http::new(config["token"]);

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
        .group(&COLOR_GROUP)
        .group(&STATISTIC_GROUP)
        .help(&MY_HELP)
        .before(before_hook)
        .after(after_hook);

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(config["token"], intents)
        .event_handler(Handler)
        .framework(framework)
        .cache_settings(|s| s.max_messages(1000))
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
        let pool = AnyPoolOptions::new()
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
