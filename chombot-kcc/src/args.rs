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

    /// Enable ranking watcher
    #[arg(long, env, default_value_t = false)]
    pub feature_ranking_watcher: bool,
    /// Ranking watcher channel ID
    #[arg(long, env)]
    pub ranking_watcher_channel_id: Option<u64>,

    /// Enable tournaments watcher
    #[arg(long, env, default_value_t = false)]
    pub feature_tournaments_watcher: bool,
    /// Tournaments watcher channel ID
    #[arg(long, env)]
    pub tournaments_watcher_channel_id: Option<u64>,

    /// Enable KCC3 features
    #[arg(long, env, default_value_t = false)]
    pub feature_kcc3: bool,
    /// KCC3 base URL
    #[arg(long, env)]
    pub kcc3_url: Option<String>,
    /// KCC3 API token
    #[arg(long, env)]
    pub kcc3_token: Option<String>,

    /// Enable pasta slash command (hermetic humour warning)
    #[arg(long, env, default_value_t = false)]
    pub feature_pasta: bool,

    /// Enable fancy text generator slash command
    #[arg(long, env, default_value_t = true)]
    pub feature_fancy_text: bool,
}
