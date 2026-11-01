use std::fmt::Write;
use std::iter;
use std::iter::Once;
use std::sync::Arc;

use async_trait::async_trait;
use log::error;
use poise::serenity_prelude::{Channel, ChannelId, Context, CreateMessage};
use tokio::sync::RwLock;

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;
use crate::data_watcher::DataUpdateNotifier;
use crate::tournaments_watcher::ema::{TournamentStatus, TournamentStatuses, Tournaments};
use crate::tournaments_watcher::updates::{
    NumberMarker, PostedTournament, PostedUpdate, TournamentUpdates, emote_numbering,
    keycap_numbering,
};

const MESSAGE_PREFIX: &str =
    "**TOURNAMENTS UPDATE** (http://mahjong-europe.org/ranking/Calendar.html)\n\n";
pub const TEST_MESSAGE_PREFIX: &str =
    "**TOURNAMENTS UPDATE** (test; http://mahjong-europe.org/ranking/Calendar.html)\n\n";

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
    updates: Arc<TournamentUpdates>,
    reactions: bool,
}

impl<T: TournamentWatcherChannelListProvider> TournamentsChannelMessageNotifier<T> {
    #[must_use]
    pub const fn new(
        channel_list_provider: T,
        updates: Arc<TournamentUpdates>,
        reactions: bool,
    ) -> Self {
        Self {
            channel_list_provider,
            updates,
            reactions,
        }
    }
}

/// Returns whether all reactions could be added.
///
/// With `reactions`, each tournament is numbered with the server's `:1m:` …
/// `:10m:` emotes (or 1️⃣ … 🔟) and gets the matching reaction, which creates
/// its channel.
pub async fn post_tournaments_update(
    ctx: &Context,
    updates: &TournamentUpdates,
    channel_id: ChannelId,
    header: &str,
    statuses: &TournamentStatuses,
    reactions: bool,
) -> anyhow::Result<bool> {
    let numbering = if reactions {
        Some(numbering_for(ctx, channel_id).await)
    } else {
        None
    };

    let mut all_reactions_added = true;
    for message in build_messages(header, statuses, numbering.as_deref()) {
        let sent = channel_id
            .send_message(ctx, CreateMessage::new().content(&message.text))
            .await?;
        let Some(numbering) = &numbering else {
            continue;
        };

        updates.record(PostedUpdate {
            message_id: sent.id,
            tournaments: message
                .statuses
                .iter()
                .map(|status| PostedTournament {
                    name: status.name().to_owned(),
                    entry: status.new_entry().cloned(),
                })
                .collect(),
        });
        for marker in &numbering[..message.statuses.len()] {
            if let Err(why) = sent.react(ctx, marker.reaction.clone()).await {
                error!("Could not add reactions to a Tournaments update: {why:?}");
                all_reactions_added = false;
                break;
            }
        }
    }

    Ok(all_reactions_added)
}

async fn numbering_for(ctx: &Context, channel_id: ChannelId) -> Vec<NumberMarker> {
    let emotes = match channel_id.to_channel(ctx).await {
        Ok(Channel::Guild(channel)) => channel.guild_id.emojis(ctx).await.unwrap_or_default(),
        _ => Vec::new(),
    };
    emote_numbering(
        emotes
            .iter()
            .filter(|emote| emote.available)
            .map(|emote| (emote.name.as_str(), emote.id, emote.animated)),
    )
    .unwrap_or_else(keycap_numbering)
}

#[derive(Default)]
struct UpdateMessage<'a> {
    text: String,
    statuses: Vec<&'a TournamentStatus>,
}

/// Splits an update into messages within Discord's size limit and, when
/// numbered, with at most one tournament per number marker.
fn build_messages<'a>(
    header: &str,
    statuses: &'a TournamentStatuses,
    numbering: Option<&[NumberMarker]>,
) -> Vec<UpdateMessage<'a>> {
    let line = |status: &TournamentStatus, index: usize| {
        let marker = numbering.map_or("*", |numbering| numbering[index].text.as_str());
        format!("{marker} {}\n", diff_as_message(status))
    };

    let mut messages = Vec::new();
    let mut current = UpdateMessage {
        text: header.to_owned(),
        statuses: Vec::new(),
    };
    for status in &statuses.0 {
        if numbering.is_some_and(|numbering| current.statuses.len() == numbering.len()) {
            messages.push(std::mem::take(&mut current));
        }
        let mut text = line(status, current.statuses.len());
        if current.text.len() + text.len() > DISCORD_MESSAGE_SIZE_LIMIT
            && !current.statuses.is_empty()
        {
            messages.push(std::mem::take(&mut current));
            text = line(status, 0);
        }
        current.text.push_str(&text);
        current.statuses.push(status);
    }
    if !current.statuses.is_empty() {
        messages.push(current);
    }

    messages
}

#[must_use]
fn diff_as_message(diff: &TournamentStatus) -> String {
    let mut str = String::new();

    match diff {
        TournamentStatus::New(entry) => {
            let _ = write!(str, "**NEW**: _{}_", entry.name);
            if !entry.url.is_empty() {
                let _ = write!(str, " ({})", entry.url);
            }
            str += "; ";
            let _ = write!(str, "{}; ", entry.date);
            let _ = write!(str, "{}; ", entry.place);
            let _ = write!(str, "MERS: {}", entry.approval_status);
            if !entry.registration_start.is_empty() {
                let _ = write!(str, "; registration start: {}", entry.registration_start);
            }
            if !entry.results_status.is_empty() {
                let _ = write!(str, "; {}", entry.results_status);
            }
        }
        TournamentStatus::Changed(change) => {
            let _ = write!(str, "**CHANGED**: _{}_; ", change.name);

            if let Some(url) = &change.url
                && !url.is_empty()
            {
                let _ = write!(str, "website: {url}; ");
            }
            if let Some(date) = &change.date {
                let _ = write!(str, "date: {date}; ");
            }
            if let Some(place) = &change.place {
                let _ = write!(str, "place: {place}; ");
            }
            if let Some(approval_status) = &change.approval_status {
                let _ = write!(str, "MERS approval: {approval_status}; ");
            }
            if let Some(registration_start) = &change.registration_start {
                let _ = write!(str, "registration start: {registration_start}; ");
            }
            if let Some(results) = &change.results_status {
                let _ = write!(str, "results: \"{results}\"; ");
            }

            {
                const SUFFIX: &str = "; ";
                if str.ends_with(SUFFIX) {
                    str.truncate(str.len() - SUFFIX.len());
                }
            }
        }
    }

    str
}

#[async_trait]
impl<T: TournamentWatcherChannelListProvider> DataUpdateNotifier<Option<Tournaments>>
    for TournamentsChannelMessageNotifier<T>
{
    async fn notify(&self, diff: TournamentStatuses, ctx: &Context) {
        let channel_list: Vec<ChannelId> = self
            .channel_list_provider
            .tournament_watcher_channels()
            .await
            .into_iter()
            .collect();
        for channel_id in channel_list {
            let result = post_tournaments_update(
                ctx,
                &self.updates,
                channel_id,
                MESSAGE_PREFIX,
                &diff,
                self.reactions,
            )
            .await;
            if let Err(why) = result {
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
    use crate::tournaments_watcher::notifier::{MESSAGE_PREFIX, build_messages};
    use crate::tournaments_watcher::updates::keycap_numbering;

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
                registration_start: None,
                results_status: Some("Results".to_owned()),
            }),
            TournamentStatus::New(TournamentEntry {
                name: "Krakow Riichi Open".to_owned(),
                url: "https://chombo.club".to_owned(),
                rules: "RCR".to_owned(),
                date: "27-31 November 2023".to_owned(),
                place: "Krakow".to_owned(),
                approval_status: "OK".to_owned(),
                registration_start: "Sept. 1st 2024".to_owned(),
                results_status: String::new(),
            }),
            TournamentStatus::Changed(TournamentChange {
                name: "ERMC 2024".to_owned(),
                url: Some("abc.com".to_owned()),
                rules: None,
                date: None,
                place: None,
                approval_status: Some("OK".to_owned()),
                registration_start: Some("Sept. 1st 2024".to_owned()),
                results_status: None,
            }),
            TournamentStatus::Changed(TournamentChange {
                name: "ERMC 2025".to_owned(),
                url: Some(String::new()),
                rules: None,
                date: None,
                place: None,
                approval_status: None,
                registration_start: None,
                results_status: None,
            }),
        ];

        let statuses = TournamentStatuses(diffs);
        let messages = build_messages(MESSAGE_PREFIX, &statuses, None);
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].text,
            format!(
                "{MESSAGE_PREFIX}{}",
                include_str!("test_data/expected_message.txt")
            )
        );
    }

    #[test]
    fn splits_long_updates() {
        let tournaments = |count: usize, name_length: usize| {
            TournamentStatuses(
                (1..=count)
                    .map(|i| {
                        TournamentStatus::New(TournamentEntry {
                            name: format!("{} {i}", "x".repeat(name_length)),
                            url: String::new(),
                            rules: "Riichi".to_owned(),
                            date: "1-2 May 2027".to_owned(),
                            place: "Krakow".to_owned(),
                            approval_status: "OK".to_owned(),
                            registration_start: String::new(),
                            results_status: String::new(),
                        })
                    })
                    .collect(),
            )
        };

        let numbering = keycap_numbering();
        let short = tournaments(12, 1);
        let numbered = build_messages(MESSAGE_PREFIX, &short, Some(&numbering));
        assert_eq!(numbered.len(), 2);
        assert_eq!(numbered[0].statuses.len(), 10);
        assert!(
            numbered[1]
                .text
                .starts_with("1\u{fe0f}\u{20e3} **NEW**: _x 11_;")
        );

        let long = tournaments(20, 150);
        let messages = build_messages(MESSAGE_PREFIX, &long, None);
        assert!(messages.len() > 1);
        assert!(
            messages
                .iter()
                .all(|message| message.text.len() <= crate::data::DISCORD_MESSAGE_SIZE_LIMIT)
        );
    }
}
