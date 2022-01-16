use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use tokio::try_join;

use crate::kcc3::data_types::{Player, PlayerId};
use crate::kcc3::Kcc3ClientError;
use crate::Kcc3Client;

#[derive(Debug)]
pub struct ChombotError {
    kcc3_client_error: Kcc3ClientError,
}

impl ChombotError {
    fn new(kcc3_client_error: Kcc3ClientError) -> Self {
        Self {
            kcc3_client_error,
        }
    }
}

impl From<Kcc3ClientError> for ChombotError {
    fn from(e: Kcc3ClientError) -> Self {
        Self::new(e)
    }
}

impl Display for ChombotError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chombot error: {}", self.kcc3_client_error)
    }
}

type ChombotResult<T> = Result<T, ChombotError>;

pub struct Chombot {
    kcc3client: Kcc3Client,
}

impl Chombot {
    pub fn new(kcc3client: Kcc3Client) -> Self {
        Self {
            kcc3client,
        }
    }

    pub async fn list_chombos_by_count(&self) -> ChombotResult<Vec<(Player, usize)>> {
        let players_fut = self.kcc3client.get_players();
        let chombos_fut = self.kcc3client.get_chombos();
        let (players, chombos) = try_join!(players_fut, chombos_fut)?;

        let mut player_map: HashMap<PlayerId, Player> = players.into_iter()
            .map(|x| (x.id.clone(), x))
            .collect();
        let mut chombo_counts: HashMap<PlayerId, usize> = HashMap::new();
        for chombo in chombos {
            let entry = chombo_counts.entry(chombo.player).or_insert(0);
            *entry += 1;
        }
        let mut result: Vec<(Player, usize)> = chombo_counts
            .into_iter()
            .map(|(player_id, num)| (player_map.remove(&player_id).unwrap(), num))
            .collect();
        result.sort_by(|(_, num_1), (_, num_2)| num_2.cmp(num_1));

        Ok(result)
    }
}
