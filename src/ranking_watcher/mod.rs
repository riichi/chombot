use std::error::Error;
use std::future::Future;

use tokio::time::{sleep, Duration};

use crate::ranking_watcher::notifier::RankingUpdateNotifier;

pub mod notifier;
pub mod usma;

const RANKING_UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub trait WatchableRanking<R> {
    fn should_notify<'a>(&self, new: &'a Self) -> Option<&'a R>;
    fn update(&mut self, new: Self);
}

impl<R: Eq> WatchableRanking<R> for Option<R> {
    fn should_notify<'a>(&self, new: &'a Self) -> Option<&'a R> {
        match (&self, &new) {
            (Some(o), Some(n)) if o != n => Some(n),
            _ => None,
        }
    }

    fn update(&mut self, new: Self) {
        if new.is_some() {
            *self = new;
        }
    }
}

pub struct RankingWatcher<R, F, H> {
    previous_ranking: Option<R>,
    update_notifier: F,
    get_next: H,
}

impl<R, F, H, HOut, E> RankingWatcher<R, F, H>
where
    R: Eq + Send + Sync,
    F: RankingUpdateNotifier<R>,
    H: Fn() -> HOut,
    HOut: Future<Output = Result<R, E>>,
    E: Error,
{
    async fn fetch_ranking(&self) -> Option<R> {
        match (self.get_next)().await {
            Ok(r) => Some(r),
            Err(e) => {
                println!("Error when fetching ranking: {:?}", e);
                None
            }
        }
    }

    pub fn new(update_notifier: F, get_next: H) -> Self {
        Self {
            previous_ranking: None,
            update_notifier,
            get_next,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let new_ranking = self.fetch_ranking().await;
            if let Some(r) = self.previous_ranking.should_notify(&new_ranking) {
                self.update_notifier.notify(r).await;
            }
            self.previous_ranking.update(new_ranking);
            sleep(RANKING_UPDATE_INTERVAL).await;
        }
    }
}
