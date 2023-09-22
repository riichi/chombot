use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use chrono::Utc;
use image::RgbaImage;
use riichi_hand::parser::{HandParseError, HandParser};
use riichi_hand::raster_renderer::fluffy_stuff_tile_sets::{
    BLACK_FLUFFY_STUFF_TILE_SET, RED_FLUFFY_STUFF_TILE_SET, YELLOW_FLUFFY_STUFF_TILE_SET,
};
use riichi_hand::raster_renderer::martin_persson_tile_sets::MARTIN_PERSSON_TILE_SET;
use riichi_hand::raster_renderer::{HandRenderError, RasterRenderer, RenderOptions, TileSet};
use tokio::try_join;

use crate::kcc3::data_types::{Chombo, Player, PlayerId};
use crate::kcc3::{Kcc3Client, Kcc3ClientError};

#[derive(Debug)]
pub enum ChombotError {
    Kcc3ClientError(Kcc3ClientError),
    HandParserError(HandParseError),
    HandRenderingError(HandRenderError),
    Kcc3ClientNotAvailable,
}

impl From<Kcc3ClientError> for ChombotError {
    fn from(e: Kcc3ClientError) -> Self {
        Self::Kcc3ClientError(e)
    }
}

impl From<HandParseError> for ChombotError {
    fn from(e: HandParseError) -> Self {
        Self::HandParserError(e)
    }
}

impl From<HandRenderError> for ChombotError {
    fn from(e: HandRenderError) -> Self {
        Self::HandRenderingError(e)
    }
}

impl Display for ChombotError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kcc3ClientError(e) => write!(f, "KCC3 client error: {e}"),
            Self::HandParserError(e) => write!(f, "Hand parse error: {e}"),
            Self::HandRenderingError(e) => write!(f, "Hand rendering error: {e}"),
            Self::Kcc3ClientNotAvailable => write!(f, "KCC3 feature disabled"),
        }
    }
}

impl Error for ChombotError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::Kcc3ClientError(e) => Some(e),
            Self::HandParserError(e) => Some(e),
            Self::HandRenderingError(e) => Some(e),
            Self::Kcc3ClientNotAvailable => None,
        }
    }
}

type ChombotResult<T> = Result<T, ChombotError>;

pub enum TileStyle {
    Yellow,
    Red,
    Black,
    MartinPersson,
}

pub struct Chombot {
    kcc3client: Option<Kcc3Client>,
}

impl Chombot {
    pub const fn new(kcc3client: Option<Kcc3Client>) -> Self {
        Self { kcc3client }
    }

    fn get_client(&self) -> ChombotResult<&Kcc3Client> {
        self.kcc3client
            .as_ref()
            .ok_or(ChombotError::Kcc3ClientNotAvailable)
    }

    pub async fn add_chombo_for_player<P, F>(
        &self,
        predicate: P,
        create_new: F,
        comment: &str,
    ) -> ChombotResult<Chombo>
    where
        P: (Fn(&Player) -> bool) + Send + Sync,
        F: (Fn() -> Player) + Send + Sync,
    {
        let client = self.get_client()?;
        let players = client.get_players().await?;
        let maybe_player = players.into_iter().find(predicate);

        let player = if let Some(player) = maybe_player {
            player
        } else {
            client.add_player(&create_new()).await?
        };

        let chombo = Chombo::new(Utc::now(), &player.id, comment);
        Ok(client.add_chombo(&chombo).await?)
    }

    pub async fn create_chombo_ranking(&self) -> ChombotResult<Vec<(Player, usize)>> {
        let client = self.get_client()?;
        let players_fut = client.get_players();
        let chombos_fut = client.get_chombos();
        let (players, chombos) = try_join!(players_fut, chombos_fut)?;

        let mut player_map: HashMap<PlayerId, Player> =
            players.into_iter().map(|x| (x.id.clone(), x)).collect();
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

    pub async fn get_chombo_list(&self) -> ChombotResult<Vec<(Player, Chombo)>> {
        let client = self.get_client()?;
        let players_fut = client.get_players();
        let chombos_fut = client.get_chombos();
        let (players, mut chombos) = try_join!(players_fut, chombos_fut)?;

        let player_map: HashMap<PlayerId, Player> =
            players.into_iter().map(|x| (x.id.clone(), x)).collect();
        chombos.sort_by_key(|chombo| chombo.timestamp);
        chombos.reverse();
        let chombos = chombos
            .into_iter()
            .map(|chombo| (player_map.get(&chombo.player).unwrap().clone(), chombo))
            .collect();

        Ok(chombos)
    }

    pub fn render_hand(hand: &str, tile_style: &TileStyle) -> ChombotResult<RgbaImage> {
        let tile_set: Box<dyn TileSet> = match tile_style {
            TileStyle::Yellow => Box::new(&*YELLOW_FLUFFY_STUFF_TILE_SET),
            TileStyle::Red => Box::new(&*RED_FLUFFY_STUFF_TILE_SET),
            TileStyle::Black => Box::new(&*BLACK_FLUFFY_STUFF_TILE_SET),
            TileStyle::MartinPersson => Box::new(&*MARTIN_PERSSON_TILE_SET),
        };

        let hand = HandParser::parse(hand)?;
        Ok(RasterRenderer::render(
            &hand,
            &tile_set,
            RenderOptions::default(),
        )?)
    }
}
