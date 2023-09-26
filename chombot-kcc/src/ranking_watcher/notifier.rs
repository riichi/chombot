use async_trait::async_trait;
use chombot_common::data_watcher::DataUpdateNotifier;
use chombot_common::discord_utils::send_with_overflow;
use log::error;
use poise::serenity_prelude::{ChannelId, Context};

use crate::ranking_watcher::usma::{PositionChangeInfo, Ranking};

pub struct ChannelMessageNotifier {
    channel_id: ChannelId,
    message: String,
}

impl ChannelMessageNotifier {
    pub const fn new(channel_id: ChannelId, message: String) -> Self {
        Self {
            channel_id,
            message,
        }
    }

    fn format_position_delta(p: &PositionChangeInfo) -> String {
        match *p {
            PositionChangeInfo::Diff(delta) => match delta {
                d if d < 0 => format!(" (↓{})", -d),
                d if d > 0 => format!(" (↑{d})"),
                _ => String::new(),
            },
            PositionChangeInfo::New => String::from(" (NEW)"),
        }
    }

    fn format_points_delta(p: &PositionChangeInfo) -> String {
        match *p {
            PositionChangeInfo::Diff(d) if d < 0 => format!(" ({d})"),
            PositionChangeInfo::Diff(d) if d > 0 => format!(" (+{d})"),
            _ => String::new(),
        }
    }

    fn build_message(&self, ranking: &Ranking) -> String {
        let mut base = self.message.clone();
        let ppl = ranking
            .get_changed()
            .into_iter()
            .map(|e| {
                format!(
                    "• {}{} / {} / {}{} pkt",
                    e.pos,
                    Self::format_position_delta(&e.pos_diff),
                    e.name,
                    e.points,
                    Self::format_points_delta(&e.points_diff)
                )
            })
            .collect::<Vec<String>>();
        base.push_str("\n\nLatest changes:\n");
        base.push_str(ppl.join("\n").as_str());
        base
    }
}

#[async_trait]
impl DataUpdateNotifier<Ranking> for ChannelMessageNotifier {
    async fn notify(&self, _old_ranking: &Ranking, new_ranking: &Ranking, ctx: &Context) {
        let channel_id = self.channel_id;
        let text = self.build_message(new_ranking);

        if let Err(why) = send_with_overflow(channel_id, ctx, &text).await {
            error!("Could not send Ranking update: {why:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ranking_watcher::usma::RankingEntry;

    #[test]
    fn test_channel_message_notifier_build_message() {
        let notifier = ChannelMessageNotifier::new(ChannelId(123), "TEST_MESSAGE".into());
        let ranking = Ranking(vec![
            RankingEntry {
                pos: 1,
                pos_diff: PositionChangeInfo::Diff(0),
                name: "player-name-001".to_owned(),
                points: 1966,
                points_diff: PositionChangeInfo::Diff(0),
            },
            RankingEntry {
                pos: 3,
                pos_diff: PositionChangeInfo::Diff(3),
                name: "player-name-002".to_owned(),
                points: 1893,
                points_diff: PositionChangeInfo::New,
            },
            RankingEntry {
                pos: 5,
                pos_diff: PositionChangeInfo::New,
                name: "player-name-003".to_owned(),
                points: 1830,
                points_diff: PositionChangeInfo::Diff(-60),
            },
            RankingEntry {
                pos: 8,
                pos_diff: PositionChangeInfo::Diff(-3),
                name: "player-name-004".to_owned(),
                points: 1718,
                points_diff: PositionChangeInfo::Diff(73),
            },
        ]);

        let message = notifier.build_message(&ranking);

        assert_eq!(
            message,
            concat!(
                "TEST_MESSAGE\n",
                "\n",
                "Latest changes:\n",
                "• 3 (↑3) / player-name-002 / 1893 pkt\n",
                "• 5 (NEW) / player-name-003 / 1830 (-60) pkt\n",
                "• 8 (↓3) / player-name-004 / 1718 (+73) pkt"
            )
        );
    }
}
