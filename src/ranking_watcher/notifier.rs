use serenity::async_trait;
use serenity::client::Context;
use serenity::model::id::ChannelId;

use crate::data_watcher::DataUpdateNotifier;
use crate::ranking_watcher::usma::{PositionChangeInfo, Ranking};

pub struct ChannelMessageNotifier {
    channel_id: ChannelId,
    ctx: Context,
    message: String,
}

impl ChannelMessageNotifier {
    pub fn new(channel_id: ChannelId, ctx: Context, message: String) -> Self {
        Self {
            channel_id,
            ctx,
            message,
        }
    }

    fn format_position_delta(p: &PositionChangeInfo) -> String {
        match *p {
            PositionChangeInfo::Diff(delta) => match delta {
                d if d < 0 => format!(" (↓{})", -d),
                d if d > 0 => format!(" (↑{d})"),
                _ => String::from(""),
            },
            PositionChangeInfo::New => String::from(" (NEW)"),
        }
    }

    fn format_points_delta(p: &PositionChangeInfo) -> String {
        match *p {
            PositionChangeInfo::Diff(d) if d < 0 => format!(" ({d})"),
            PositionChangeInfo::Diff(d) if d > 0 => format!(" (+{d})"),
            _ => String::from(""),
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
    async fn notify(&self, _old_ranking: &Ranking, new_ranking: &Ranking) {
        self.channel_id
            .send_message(&self.ctx, |m| {
                m.content(self.build_message(new_ranking).as_str())
            })
            .await
            .unwrap();
    }
}
