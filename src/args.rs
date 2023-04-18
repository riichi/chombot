use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    /// Discord API token
    #[arg(long, env)]
    pub discord_token: String,
    /// Discord Application ID
    #[arg(long, env)]
    pub application_id: u64,
    /// Guild (Discord server) ID
    #[arg(long, env)]
    pub guild_id: u64,

    /// Enable ranking watcher
    #[arg(long, env, default_value_t = false)]
    pub feature_ranking_watcher: bool,
    /// Ranking watcher channel ID
    #[arg(long, env)]
    pub ranking_watcher_channel_id: Option<u64>,

    /// Enable KCC3 features
    #[arg(long, env, default_value_t = false)]
    pub feature_kcc3: bool,
    /// KCC3 base URL
    #[arg(long, env)]
    pub kcc3_url: Option<String>,
    /// KCC3 API token
    #[arg(long, env)]
    pub kcc3_token: Option<String>,

    /// Enable pasta slash command (hermeric humour warning)
    #[arg(long, env, default_value_t = false)]
    pub feature_pasta: bool,
}
