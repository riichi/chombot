use std::error::Error;
use std::fmt::{Display, Formatter};

use image::RgbaImage;
use riichi_hand::parser::{HandParseError, HandParser};
use riichi_hand::raster_renderer::fluffy_stuff_tile_sets::{
    BLACK_FLUFFY_STUFF_TILE_SET, RED_FLUFFY_STUFF_TILE_SET, YELLOW_FLUFFY_STUFF_TILE_SET,
};
use riichi_hand::raster_renderer::martin_persson_tile_sets::MARTIN_PERSSON_TILE_SET;
use riichi_hand::raster_renderer::{HandRenderError, RasterRenderer, RenderOptions, TileSet};

#[derive(Debug)]
pub enum ChombotBaseError {
    HandParserError(HandParseError),
    HandRenderingError(HandRenderError),
}

impl From<HandParseError> for ChombotBaseError {
    fn from(e: HandParseError) -> Self {
        Self::HandParserError(e)
    }
}

impl From<HandRenderError> for ChombotBaseError {
    fn from(e: HandRenderError) -> Self {
        Self::HandRenderingError(e)
    }
}

impl Display for ChombotBaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HandParserError(e) => write!(f, "Hand parse error: {e}"),
            Self::HandRenderingError(e) => write!(f, "Hand rendering error: {e}"),
        }
    }
}

impl Error for ChombotBaseError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::HandParserError(e) => Some(e),
            Self::HandRenderingError(e) => Some(e),
        }
    }
}

type ChombotResult<T> = Result<T, ChombotBaseError>;

pub enum TileStyle {
    Yellow,
    Red,
    Black,
    MartinPersson,
}

pub struct ChombotBase {}

impl ChombotBase {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
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
