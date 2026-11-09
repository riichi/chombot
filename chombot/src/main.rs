#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use anyhow::Error;
use chombot_common::chombot::ChombotBase;
use chombot_common::slash_commands::create_channel::create_channel;
use chombot_common::slash_commands::hand::hand;
use chombot_common::slash_commands::score::score;
use chombot_common::slash_commands::tournament_update_test::tournament_update_test;
use chombot_common::tournament_reactions::handle_reaction_add;
use chombot_common::tournaments_watcher::updates::TournamentUpdates;
use chombot_common::{ChombotPoiseUserData, start_tournaments_watcher};
use clap::Parser;
use log::{LevelFilter, error, info};
use poise::serenity_prelude::{ClientBuilder, FullEvent, GatewayIntents};
use poise::{BoxFuture, Command, Context, Framework, FrameworkContext, FrameworkOptions};
use tokio::sync::RwLock;

use crate::args::Arguments;
use crate::config::ChombotConfig;
use crate::tournament_watcher::tournament_watcher;

mod args;
mod config;
mod tournament_watcher;

pub struct PoiseUserData {
    pub chombot: ChombotBase,
    pub config: Arc<RwLock<ChombotConfig>>,
}

impl ChombotPoiseUserData for PoiseUserData {
    fn chombot(&self) -> &ChombotBase {
        &self.chombot
    }
}

pub type PoiseContext<'a> = Context<'a, PoiseUserData, anyhow::Error>;

fn get_command_list() -> Vec<Command<PoiseUserData, Error>> {
    vec![
        hand(),
        score(),
        tournament_watcher(),
        create_channel(),
        tournament_update_test(),
    ]
}

const CONFIG_DATA_PATH: &str = "data.toml";
const TOURNAMENT_UPDATES_PATH: &str = "tournament_updates.json";

fn event_handler<'a>(
    framework_ctx: FrameworkContext<'a, PoiseUserData, Error>,
    event: &'a FullEvent,
) -> BoxFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
        if let FullEvent::ReactionAdd { add_reaction } = event {
            handle_reaction_add(
                framework_ctx.serenity_context,
                add_reaction,
                framework_ctx.user_data,
                framework_ctx.bot_id(),
            )
            .await?;
        }
        Ok(())
    })
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("chombot", LevelFilter::Info)
        .init();

    let args = Arguments::parse();
    let tournament_updates = Arc::new(TournamentUpdates::load(TOURNAMENT_UPDATES_PATH));
    let chombot = ChombotBase::new(tournament_updates.clone());
    let config = ChombotConfig::load(CONFIG_DATA_PATH.into()).expect("Could not load config");
    let config_ref = Arc::new(RwLock::new(config));

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: get_command_list(),
            event_handler,
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                start_tournaments_watcher(
                    config_ref.clone(),
                    tournament_updates,
                    true,
                    ctx.clone(),
                    chombot.tournaments().clone(),
                );
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("{} is connected!", ready.user.name);
                Ok(PoiseUserData {
                    chombot,
                    config: config_ref,
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(&args.discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Could not create client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
