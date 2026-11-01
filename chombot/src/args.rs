#![allow(clippy::struct_excessive_bools)]

use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    /// Discord API token
    #[arg(long, env)]
    pub discord_token: String,

    /// Enable archiving channels by writing `#archiwum` (needs the Message
    /// Content intent)
    #[arg(long, env, default_value_t = false)]
    pub feature_archive_votes: bool,
}
