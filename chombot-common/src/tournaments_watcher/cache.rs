use std::sync::Arc;
use std::time::{Duration, Instant};

use log::warn;
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::tournaments_watcher::ema::{Tournaments, TournamentsFetchError, get_tournaments};

const CACHE_TTL: Duration = Duration::from_mins(10);

/// The EMA RCR tournament list, fetched on demand and cached for
/// [`CACHE_TTL`].
#[derive(Default)]
pub struct TournamentsCache {
    cached: Mutex<Option<(Instant, Arc<Tournaments>)>>,
    /// Held while fetching, so that only one fetch runs at a time and the
    /// cached list stays readable meanwhile.
    refreshing: Mutex<()>,
}

impl TournamentsCache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached: Mutex::const_new(None),
            refreshing: Mutex::const_new(()),
        }
    }

    /// Falls back to a stale list if refreshing fails.
    pub async fn get(&self) -> Result<Arc<Tournaments>, TournamentsFetchError> {
        if let Some(tournaments) = self.fresh().await {
            return Ok(tournaments);
        }
        let _refreshing = self.refreshing.lock().await;
        if let Some(tournaments) = self.fresh().await {
            return Ok(tournaments);
        }

        match get_tournaments().await {
            Ok(tournaments) => {
                let tournaments = Arc::new(tournaments.into_rcr_only());
                *self.cached.lock().await = Some((Instant::now(), tournaments.clone()));
                Ok(tournaments)
            }
            Err(err) => match self.last_fetched().await {
                Some(stale) => {
                    warn!("Could not refresh the tournament list, using stale data: {err}");
                    Ok(stale)
                }
                None => Err(err),
            },
        }
    }

    /// Like [`Self::get`], but gives up (logging why) after `limit`, falling
    /// back to a stale list if there is one.
    pub async fn get_within(&self, limit: Duration) -> Option<Arc<Tournaments>> {
        match timeout(limit, self.get()).await {
            Ok(Ok(tournaments)) => Some(tournaments),
            Ok(Err(err)) => {
                warn!("{err}");
                None
            }
            Err(_) => {
                warn!("Timed out fetching the tournament list");
                self.last_fetched().await
            }
        }
    }

    async fn fresh(&self) -> Option<Arc<Tournaments>> {
        self.cached
            .lock()
            .await
            .as_ref()
            .filter(|(fetched_at, _)| fetched_at.elapsed() < CACHE_TTL)
            .map(|(_, tournaments)| tournaments.clone())
    }

    async fn last_fetched(&self) -> Option<Arc<Tournaments>> {
        self.cached
            .lock()
            .await
            .as_ref()
            .map(|(_, tournaments)| tournaments.clone())
    }
}
