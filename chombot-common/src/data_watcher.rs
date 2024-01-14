use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use log::error;
use poise::serenity_prelude::Context;
use tokio::time::sleep;

const DATA_UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub trait WatchableData {
    type Diff;

    #[must_use]
    fn should_notify<'a>(&'a self, new: &'a Self) -> Option<Self::Diff>;
    fn update(&mut self, new: Self);
}

impl<T: WatchableData> WatchableData for Option<T> {
    type Diff = T::Diff;

    #[must_use]
    fn should_notify<'a>(&'a self, new: &'a Self) -> Option<T::Diff> {
        match (&self, &new) {
            (Some(o), Some(n)) => o.should_notify(n),
            _ => None,
        }
    }

    fn update(&mut self, new: Self) {
        match (self.as_mut(), new) {
            (None, new) => *self = new,
            (Some(_), None) => (),
            (Some(old), Some(new)) => old.update(new),
        }
    }
}

pub struct DataWatcher<T, F, H> {
    previous_data: T,
    update_notifier: F,
    get_next: H,
}

impl<T, F, H, HOut, E> DataWatcher<T, F, H>
where
    T: WatchableData + Send + Sync + Default,
    T::Diff: Send + Sync,
    F: DataUpdateNotifier<T> + Send + Sync,
    H: (Fn() -> HOut) + Send + Sync,
    HOut: Future<Output = Result<T, E>> + Send + Sync,
    E: Debug,
{
    #[must_use]
    async fn fetch_data(&self) -> Option<T> {
        match (self.get_next)().await {
            Ok(r) => Some(r),
            Err(e) => {
                error!("Error when fetching data: {e:?}");
                None
            }
        }
    }

    #[must_use]
    pub fn new(update_notifier: F, get_next: H) -> Self {
        Self {
            previous_data: Default::default(),
            update_notifier,
            get_next,
        }
    }

    pub async fn run(&mut self, ctx: &Context) {
        loop {
            sleep(DATA_UPDATE_INTERVAL).await;
            if let Some(new_data) = self.fetch_data().await {
                if let Some(diff) = self.previous_data.should_notify(&new_data) {
                    self.update_notifier.notify(diff, ctx).await;
                }
                self.previous_data.update(new_data);
            }
        }
    }
}

#[async_trait]
pub trait DataUpdateNotifier<T: WatchableData + Send + Sync> {
    async fn notify(&self, diff: T::Diff, ctx: &Context);
}
