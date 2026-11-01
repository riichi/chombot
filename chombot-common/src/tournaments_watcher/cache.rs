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
}

impl TournamentsCache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached: Mutex::const_new(None),
        }
    }

    /// Falls back to a stale list if refreshing fails.
    pub async fn get(&self) -> Result<Arc<Tournaments>, TournamentsFetchError> {
        let mut cached = self.cached.lock().await;
        if let Some((fetched_at, tournaments)) = cached.as_ref()
            && fetched_at.elapsed() < CACHE_TTL
        {
            return Ok(tournaments.clone());
        }

        match get_tournaments().await {
            Ok(tournaments) => {
                let tournaments = Arc::new(tournaments.into_rcr_only());
                *cached = Some((Instant::now(), tournaments.clone()));
                Ok(tournaments)
            }
            Err(err) => match cached.as_ref() {
                Some((_, stale)) => {
                    warn!("Could not refresh the tournament list, using stale data: {err}");
                    Ok(stale.clone())
                }
                None => Err(err),
            },
        }
    }

    /// Like [`Self::get`], but gives up (logging why) after `limit`.
    pub async fn get_within(&self, limit: Duration) -> Option<Arc<Tournaments>> {
        match timeout(limit, self.get()).await {
            Ok(Ok(tournaments)) => Some(tournaments),
            Ok(Err(err)) => {
                warn!("{err}");
                None
            }
            Err(_) => {
                warn!("Timed out fetching the tournament list");
                None
            }
        }
    }
}
