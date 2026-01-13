#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use poise::serenity_prelude::Context as SerenityContext;
use poise::Context;

use crate::chombot::ChombotBase;
use crate::data_watcher::DataWatcher;
use crate::tournaments_watcher::ema::get_rcr_tournaments;
use crate::tournaments_watcher::notifier::{
    TournamentWatcherChannelListProvider, TournamentsChannelMessageNotifier,
};

pub mod chombot;
pub mod data;
pub mod data_watcher;
pub mod discord_utils;
pub mod ranking_watcher;
pub mod scraping_utils;
pub mod slash_commands;
pub mod tournaments_watcher;

pub trait ChombotPoiseUserData: Sync {
    fn chombot(&self) -> &ChombotBase;
}

pub type ChombotPoiseContext<'a, T> = Context<'a, T, anyhow::Error>;

pub fn start_tournaments_watcher<T: TournamentWatcherChannelListProvider + 'static>(
    channel_list_provider: T,
    ctx: SerenityContext,
) {
    let notifier = TournamentsChannelMessageNotifier::new(channel_list_provider);
    tokio::spawn(async move {
        DataWatcher::new(notifier, get_rcr_tournaments)
            .run(&ctx)
            .await;
    });
}
