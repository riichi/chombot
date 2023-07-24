use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use log::error;
use poise::serenity_prelude::Context;
use tokio::time::sleep;

const DATA_UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 10);

pub trait WatchableData<T> {
    #[must_use]
    fn should_notify<'a>(&'a self, new: &'a Self) -> Option<(&'a T, &'a T)>;
    fn update(&mut self, new: Self);
}

impl<T: Eq> WatchableData<T> for Option<T> {
    #[must_use]
    fn should_notify<'a>(&'a self, new: &'a Self) -> Option<(&'a T, &'a T)> {
        match (&self, &new) {
            (Some(o), Some(n)) if o != n => Some((o, n)),
            _ => None,
        }
    }

    fn update(&mut self, new: Self) {
        if new.is_some() {
            *self = new;
        }
    }
}

pub struct DataWatcher<T, F, H> {
    previous_data: Option<T>,
    update_notifier: F,
    get_next: H,
}

impl<T, F, H, HOut, E> DataWatcher<T, F, H>
where
    T: Eq + Send + Sync,
    F: DataUpdateNotifier<T>,
    H: Fn() -> HOut,
    HOut: Future<Output = Result<T, E>>,
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
            previous_data: None,
            update_notifier,
            get_next,
        }
    }

    pub async fn run(&mut self, ctx: &Context) {
        loop {
            let new_data = self.fetch_data().await;
            if let Some((old, new)) = self.previous_data.should_notify(&new_data) {
                self.update_notifier.notify(old, new, ctx).await;
            }
            self.previous_data.update(new_data);
            sleep(DATA_UPDATE_INTERVAL).await;
        }
    }
}

#[async_trait]
pub trait DataUpdateNotifier<T: Send + Sync> {
    async fn notify(&self, old_data: &T, new_data: &T, ctx: &Context);
}
