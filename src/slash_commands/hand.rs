use std::io::Cursor;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use image::DynamicImage;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::channel::AttachmentType;

use crate::chombot::TileStyle;
use crate::slash_commands::utils::get_string_option;
use crate::slash_commands::SlashCommand;
use crate::Chombot;

const HAND_COMMAND: &str = "hand";
const HAND_OPTION: &str = "hand";
const TILE_STYLE_OPTION: &str = "tileset";

const YELLOW_TILE_SET: &str = "yellow";
const RED_TILE_SET: &str = "red";
const BLACK_TILE_SET: &str = "black";
const MARTIN_PERSSON_TILE_SET: &str = "martin-persson";
const DEFAULT_TILE_SET: &str = YELLOW_TILE_SET;

const MAX_HAND_LENGTH: u16 = 150;

pub struct HandCommand;

impl HandCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SlashCommand for HandCommand {
    fn get_name(&self) -> &'static str {
        HAND_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Draw a specified hand")
            .create_option(|option| {
                option
                    .name(HAND_OPTION)
                    .description("The hand to render")
                    .kind(CommandOptionType::String)
                    .max_length(MAX_HAND_LENGTH)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(TILE_STYLE_OPTION)
                    .description("Tile style")
                    .kind(CommandOptionType::String)
                    .add_string_choice("Yellow", YELLOW_TILE_SET)
                    .add_string_choice("Red", RED_TILE_SET)
                    .add_string_choice("Black", BLACK_TILE_SET)
                    .add_string_choice("Martin Persson", MARTIN_PERSSON_TILE_SET)
                    .required(false)
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        chombot: &Chombot,
    ) -> Result<()> {
        let hand = get_string_option(&command.data.options, HAND_OPTION)
            .ok_or(anyhow!("Missing hand description"))?;
        let tile_set =
            get_string_option(&command.data.options, TILE_STYLE_OPTION).unwrap_or(DEFAULT_TILE_SET);
        let render_tile_set = match tile_set {
            YELLOW_TILE_SET => Ok(TileStyle::Yellow),
            RED_TILE_SET => Ok(TileStyle::Red),
            BLACK_TILE_SET => Ok(TileStyle::Black),
            MARTIN_PERSSON_TILE_SET => Ok(TileStyle::MartinPersson),
            _ => Err(anyhow!("Invalid tile set: {tile_set}")),
        }?;

        let image = chombot.render_hand(hand, render_tile_set).await?;
        let mut buf = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)?;

        let files: Vec<AttachmentType> = vec![(buf.as_slice(), "hand.png").into()];
        command
            .channel_id
            .send_files(&ctx.http, files, |m| m)
            .await?;

        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content("<:Ichiwink:591396074141515776>")
            })
            .await?;

        Ok(())
    }
}
