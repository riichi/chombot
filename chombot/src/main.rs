#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use anyhow::Error;
use chombot_common::chombot::ChombotBase;
use chombot_common::slash_commands::hand::hand;
use chombot_common::slash_commands::score::score;
use chombot_common::{start_tournaments_watcher, ChombotPoiseUserData};
use clap::Parser;
use log::{error, info, LevelFilter};
use poise::serenity_prelude::GatewayIntents;
use poise::{Command, Context, Framework, FrameworkOptions};
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
    vec![hand(), score(), tournament_watcher()]
}

const CONFIG_DATA_PATH: &str = "data.toml";

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("chombot", LevelFilter::Info)
        .init();

    let args = Arguments::parse();
    let chombot = ChombotBase::new();
    let config = ChombotConfig::load(CONFIG_DATA_PATH.into()).expect("Could not load config");
    let config_ref = Arc::new(RwLock::new(config));

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: get_command_list(),
            ..Default::default()
        })
        .token(&args.discord_token)
        .intents(GatewayIntents::non_privileged())
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                start_tournaments_watcher(config_ref.clone(), ctx.clone());
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("{} is connected!", ready.user.name);
                Ok(PoiseUserData {
                    chombot,
                    config: config_ref,
                })
            })
        });

    if let Err(why) = framework.run().await {
        error!("Client error: {why:?}");
    }
}
