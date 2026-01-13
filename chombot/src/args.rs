#![allow(clippy::struct_excessive_bools)]

use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    /// Discord API token
    #[arg(long, env)]
    pub discord_token: String,

    /// Enable ranking watcher
    #[arg(long, env, default_value_t = false)]
    pub feature_ranking_watcher: bool,
}
