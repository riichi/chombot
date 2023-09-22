#![allow(clippy::struct_excessive_bools)]

use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    /// Discord API token
    #[arg(long, env)]
    pub discord_token: String,
    /// Guild (Discord server) ID
    #[arg(long, env)]
    pub guild_id: u64,

    /// Enable tournaments watcher
    #[arg(long, env, default_value_t = false)]
    pub feature_tournaments_watcher: bool,
    /// Tournaments watcher channel ID
    #[arg(long, env)]
    pub tournaments_watcher_channel_id: Option<u64>,
}
