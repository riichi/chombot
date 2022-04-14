use std::future::Future;

use tokio::time::{sleep, Duration};

use crate::ranking_watcher::notifier::RankingUpdateNotifier;
use crate::ranking_watcher::usma::Ranking;

pub mod notifier;
pub mod usma;

const RANKING_UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub trait WatchableRanking {
    fn should_notify(&self, new: &Self) -> bool;
}

impl WatchableRanking for Option<Ranking> {
    fn should_notify(&self, new: &Self) -> bool {
        match &new {
            None => false,
            Some(n) => match &self {
                None => true,
                Some(o) => n != o,
            },
        }
    }
}

pub struct RankingWatcher<R, F, H> {
    previous_ranking: R,
    update_notifier: F,
    get_next: H,
}

impl<R, F, H, HOut> RankingWatcher<R, F, H>
where
    R: WatchableRanking + Send + Sync + Default,
    F: RankingUpdateNotifier<R>,
    H: Fn() -> HOut,
    HOut: Future<Output = R>,
{
    pub fn new(update_notifier: F, get_next: H) -> Self {
        Self {
            previous_ranking: Default::default(),
            update_notifier,
            get_next,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let new_ranking = (self.get_next)().await;
            if self.previous_ranking.should_notify(&new_ranking) {
                self.update_notifier.notify(&new_ranking).await;
            }
            self.previous_ranking = new_ranking;
            sleep(RANKING_UPDATE_INTERVAL).await;
        }
    }
}
