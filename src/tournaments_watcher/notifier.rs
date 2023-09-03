use async_trait::async_trait;
use log::error;
use poise::serenity_prelude::{ChannelId, Context};

use crate::data_watcher::DataUpdateNotifier;
use crate::discord_utils::send_with_overflow;
use crate::tournaments_watcher::ema::{
    tournaments_diff, TournamentStatus, TournamentStatuses, Tournaments,
};

#[async_trait]
pub trait TournamentsUpdateNotifier<R: Send + Sync> {
    async fn notify(&self, ranking: &R);
}

pub struct TournamentsChannelMessageNotifier {
    channel_id: ChannelId,
    message: String,
}

impl TournamentsChannelMessageNotifier {
    #[must_use]
    pub const fn new(channel_id: ChannelId, message: String) -> Self {
        Self {
            channel_id,
            message,
        }
    }

    #[must_use]
    fn build_message(&self, tournament_statuses: &TournamentStatuses) -> String {
        format!("{}{}", self.message, build_message(tournament_statuses))
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

            if let Some(trimmed) = str.strip_suffix("; ") {
                str = trimmed.to_owned();
            }
        }
    };

    str
}

#[async_trait]
impl DataUpdateNotifier<Tournaments> for TournamentsChannelMessageNotifier {
    async fn notify(
        &self,
        old_tournaments: &Tournaments,
        new_tournaments: &Tournaments,
        ctx: &Context,
    ) {
        let diff = tournaments_diff(old_tournaments, new_tournaments);

        let channel_id = self.channel_id;
        let text = self.build_message(&diff);

        if let Err(why) = send_with_overflow(channel_id, ctx, text).await {
            error!("Could not send Tournaments update: {why:?}");
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
