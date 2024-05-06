use std::iter;
use std::iter::Once;
use std::sync::Arc;

use async_trait::async_trait;
use log::error;
use poise::serenity_prelude::{ChannelId, Context};
use tokio::sync::RwLock;

use crate::data_watcher::DataUpdateNotifier;
use crate::discord_utils::send_with_overflow;
use crate::tournaments_watcher::ema::{TournamentStatus, TournamentStatuses, Tournaments};

const MESSAGE_PREFIX: &str =
    "**TOURNAMENTS UPDATE** (http://mahjong-europe.org/ranking/Calendar.html)\n\n";

#[async_trait]
pub trait TournamentsUpdateNotifier<R: Send + Sync> {
    async fn notify(&self, ranking: &R);
}

#[async_trait]
pub trait TournamentWatcherChannelListProvider: Send + Sync {
    type TournamentWatcherChannelList: IntoIterator<Item = ChannelId> + Send;

    async fn tournament_watcher_channels(&self) -> Self::TournamentWatcherChannelList;
}

#[async_trait]
impl TournamentWatcherChannelListProvider for ChannelId {
    type TournamentWatcherChannelList = Once<Self>;

    async fn tournament_watcher_channels(&self) -> Self::TournamentWatcherChannelList {
        iter::once(*self)
    }
}

#[async_trait]
impl<T: TournamentWatcherChannelListProvider> TournamentWatcherChannelListProvider
    for Arc<RwLock<T>>
{
    type TournamentWatcherChannelList = T::TournamentWatcherChannelList;

    async fn tournament_watcher_channels(&self) -> Self::TournamentWatcherChannelList {
        self.read().await.tournament_watcher_channels().await
    }
}

pub struct TournamentsChannelMessageNotifier<T> {
    channel_list_provider: T,
}

impl<T: TournamentWatcherChannelListProvider> TournamentsChannelMessageNotifier<T> {
    #[must_use]
    pub const fn new(channel_list_provider: T) -> Self {
        Self {
            channel_list_provider,
        }
    }

    #[must_use]
    fn build_message(tournament_statuses: &TournamentStatuses) -> String {
        format!("{}{}", MESSAGE_PREFIX, build_message(tournament_statuses))
    }
}

#[must_use]
fn build_message(tournaments: &TournamentStatuses) -> String {
    let mut str = String::new();
    for diff in &tournaments.0 {
        str += &format!("* {}\n", diff_as_message(diff));
    }

    str
}

#[must_use]
fn diff_as_message(diff: &TournamentStatus) -> String {
    let mut str = String::new();

    match diff {
        TournamentStatus::New(entry) => {
            str += "**NEW**: ";
            str += &format!("_{}_", entry.name);
            if !entry.url.is_empty() {
                str += &format!(" ({})", entry.url);
            }
            str += "; ";
            str += &format!("{}; ", entry.date);
            str += &format!("{}; ", entry.place);
            str += &format!("MERS: {}", entry.approval_status);
            if !entry.results_status.is_empty() {
                str += &format!("; {}", entry.results_status);
            }
        }
        TournamentStatus::Changed(change) => {
            str += &format!("**CHANGED**: _{}_; ", change.name);

            if let Some(date) = &change.date {
                str += &format!("date: {date}; ");
            }
            if let Some(place) = &change.place {
                str += &format!("place: {place}; ");
            }
            if let Some(approval_status) = &change.approval_status {
                str += &format!("MERS approval: {approval_status}; ");
            }
            if let Some(results) = &change.results_status {
                str += &format!("results: \"{results}\"; ");
            }

            {
                const SUFFIX: &str = "; ";
                if str.ends_with(SUFFIX) {
                    str.truncate(str.len() - SUFFIX.len());
                }
            }
        }
    };

    str
}

#[async_trait]
impl<T: TournamentWatcherChannelListProvider> DataUpdateNotifier<Option<Tournaments>>
    for TournamentsChannelMessageNotifier<T>
{
    async fn notify(&self, diff: TournamentStatuses, ctx: &Context) {
        let text = Self::build_message(&diff);

        let channel_list: Vec<ChannelId> = self
            .channel_list_provider
            .tournament_watcher_channels()
            .await
            .into_iter()
            .collect();
        for channel_id in channel_list {
            if let Err(why) = send_with_overflow(channel_id, ctx, &text).await {
                error!("Could not send Tournaments update: {why:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tournaments_watcher::ema::{
        TournamentChange, TournamentEntry, TournamentStatus, TournamentStatuses,
    };
    use crate::tournaments_watcher::notifier::build_message;

    #[test]
    fn test() {
        let diffs = vec![
            TournamentStatus::Changed(TournamentChange {
                name: "Poteto Riichi Taikai 2023".to_owned(),
                url: None,
                rules: None,
                date: Some("1-2 November 2023".to_owned()),
                place: None,
                approval_status: None,
                results_status: Some("Results".to_owned()),
            }),
            TournamentStatus::New(TournamentEntry {
                name: "Krakow Riichi Open".to_owned(),
                url: "https://chombo.club".to_owned(),
                rules: "RCR".to_owned(),
                date: "27-31 November 2023".to_owned(),
                place: "Krakow".to_owned(),
                approval_status: "OK".to_owned(),
                results_status: String::new(),
            }),
        ];

        assert_eq!(
            build_message(&TournamentStatuses(diffs)),
            include_str!("test_data/expected_message.txt")
        );
    }
}
