use std::io::Cursor;

use anyhow::Result;
use poise::serenity_prelude::{CreateAttachment, CreateMessage};
use poise::ChoiceParameter;
use riichi_hand::image::{DynamicImage, ImageFormat};

use crate::chombot::{ChombotBase, TileStyle};
use crate::{ChombotPoiseContext, ChombotPoiseUserData};

#[derive(Debug, ChoiceParameter)]
pub enum Tileset {
    Yellow,
    Red,
    Black,
    #[name = "Martin Persson"]
    MartinPersson,
}

impl Default for Tileset {
    fn default() -> Self {
        Self::Yellow
    }
}

impl From<Tileset> for TileStyle {
    fn from(value: Tileset) -> Self {
        match value {
            Tileset::Yellow => Self::Yellow,
            Tileset::Red => Self::Red,
            Tileset::Black => Self::Black,
            Tileset::MartinPersson => Self::MartinPersson,
        }
    }
}

/// Draw a specified hand.
#[poise::command(slash_command)]
pub async fn hand<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "The hand to render"]
    #[max_length = 150]
    hand: String,
    #[description = "Tile style"] tileset: Option<Tileset>,
) -> Result<()> {
    let tile_style: TileStyle = tileset.unwrap_or_default().into();

    let image = ChombotBase::render_hand(&hand, &tile_style)?;
    let mut buf = Vec::new();
    DynamicImage::ImageRgba8(image).write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;

    let files: Vec<CreateAttachment> = vec![CreateAttachment::bytes(buf.as_slice(), "hand.png")];
    ctx.channel_id()
        .send_files(&ctx.http(), files, CreateMessage::new())
        .await?;

    ctx.say("<:Ichiwink:591396074141515776>").await?;

    Ok(())
}
