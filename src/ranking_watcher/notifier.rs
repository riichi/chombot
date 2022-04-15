use serenity::{async_trait, client::Context, model::id::ChannelId};

use crate::ranking_watcher::usma::{PositionChangeInfo, Ranking};

#[async_trait]
pub trait RankingUpdateNotifier<R: Send + Sync> {
    async fn notify(&self, ranking: &R);
}

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

    fn format_delta(p: &PositionChangeInfo) -> String {
        match *p {
            PositionChangeInfo::Diff(delta) => match delta {
                d if d < 0 => format!("({})", d),
                d if d > 0 => format!("(+{})", d),
                _ => String::from(""),
            },
            PositionChangeInfo::New => String::from("(NEW)"),
        }
    }

    fn position_changed(p: &PositionChangeInfo) -> bool {
        match *p {
            PositionChangeInfo::Diff(x) if x != 0 => true,
            PositionChangeInfo::New => true,
            _ => false,
        }
    }

    fn build_message(&self, ranking: &Ranking) -> String {
        let mut base = self.message.clone();
        let ppl = ranking
            .iter()
            .filter(|entry| {
                Self::position_changed(&entry.pos_diff)
                    || Self::position_changed(&entry.points_diff)
            })
            .map(|e| {
                format!(
                    "• {}{} / {} / {}{} pkt",
                    e.pos,
                    Self::format_delta(&e.pos_diff),
                    e.name,
                    e.points,
                    Self::format_delta(&e.points_diff)
                )
            })
            .collect::<Vec<String>>();
        base.push_str("\n\n");
        base.push_str(ppl.join("\n").as_str());
        base
    }
}

#[async_trait]
impl RankingUpdateNotifier<Ranking> for ChannelMessageNotifier {
    async fn notify(&self, ranking: &Ranking) {
        self.channel_id
            .send_message(&self.ctx, |m| {
                m.content(self.build_message(ranking).as_str())
            })
            .await
            .unwrap();
    }
}
