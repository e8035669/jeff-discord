use anyhow::{Error, Result};
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use jeff_discord::*;
use migration::{Migrator, MigratorTrait};
use poise::serenity_prelude::{self as serenity, FullEvent, GatewayIntents};
use poise::FrameworkContext;
use sea_orm::Database;
use std::convert::From;
use std::time::Duration;
use std::{collections::HashMap, env, sync::Arc};
use tracing::{error, warn};
use tracing_subscriber::filter::LevelFilter;

struct MyCacheAndHttp {
    cache: Arc<serenity::Cache>,
    http: Arc<serenity::Http>,
}

impl From<&serenity::Context> for MyCacheAndHttp {
    fn from(_ctx: &serenity::Context) -> Self {
        Self {
            cache: _ctx.cache.clone(),
            http: _ctx.http.clone(),
        }
    }
}

impl serenity::CacheHttp for MyCacheAndHttp {
    fn http(&self) -> &serenity::Http {
        &self.http
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}

#[poise::command(prefix_command, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "嗨 我是薏仁的機器人"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<()> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            show_context_menu_commands: true,
            ephemeral: false,
            ..Default::default()
        },
    ).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            warn!("Error in command `{}`: {:?}", ctx.command().name, error);
            let ret = ctx.reply(format!("Command `{}` execution error", ctx.command().name)).await;
            if let Err(e) = ret {
                warn!("Error while sending error reply {:?}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                warn!("Error while handling error: {}", e);
            }
        }
    }
}

async fn on_event(
    ctx: &serenity::Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<()> {
    println!(
        "Got an event in event handler: {:?}",
        event.snake_case_name()
    );
    match event {
        FullEvent::Message { .. } => on_message(ctx, event, framework, data).await?,
        _ => {}
    }

    Ok(())
}

async fn on_message(
    ctx: &serenity::Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<()> {
    let message = match event {
        FullEvent::Message { new_message } => Ok(new_message),
        _ => Err(BotError::MessageNotFromGuild),
    }?;

    let handlers = vec![handle_message];
    for h in &handlers {
        if let Err(e) = h(ctx, event, framework, data, message).await {
            error!("Handle on_message error {:?}", e);
        }
    }

    Ok(())
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

async fn run_bot(config: HashMap<&str, &str>) {
    let token = config["token"].to_owned();
    let database_url = config["database_url"].to_owned();

    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(LevelFilter::WARN.into())
        .add_directive("jeff_discord=DEBUG".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            register(),
            colorreg(),
            colorunreg(),
            nextcolor(),
            listregs(),
            botsend(),
            ping(),
            emojistat(),
            role_move(),
            role_show(),
            chat(),
            write(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("$".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|_ctx| Box::pin(async move { Ok(true) })),
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(on_event(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                let pool = Database::connect(database_url.as_str()).await?;
                Migrator::up(&pool, None).await?;

                let color_data = Arc::new(ColorRandom::new(pool.clone()));
                let openai_client = Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base("http://192.168.17.20:8000/v1")
                        .with_api_key("EMPTY"),
                );
                let data = Data {
                    pool,
                    color_data: color_data.clone(),
                    openai: openai_client,
                };

                _ctx.set_activity(Some(serenity::ActivityData::listening("夏天的蟬鳴")));

                let cachehttp = MyCacheAndHttp::from(_ctx);

                tokio::spawn(async move {
                    let color_data = color_data;
                    update_loop(&cachehttp, &color_data).await;
                });

                poise::builtins::register_globally(_ctx, &_framework.options().commands).await?;

                Ok(data)
            })
        })
        .options(options)
        .build();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
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
