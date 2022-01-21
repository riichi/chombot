use async_trait::async_trait;
use image::DynamicImage;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::AttachmentType;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandOptionType,
};
use serenity::model::interactions::InteractionResponseType;

use crate::chombot::TileStyle;
use crate::slash_commands::SlashCommand;
use crate::Chombot;

const HAND_COMMAND: &'static str = "hand";
const HAND_OPTION: &'static str = "hand";
const TILE_STYLE_OPTION: &'static str = "tileset";

const YELLOW_TILE_SET: &'static str = "yellow";
const RED_TILE_SET: &'static str = "red";
const BLACK_TILE_SET: &'static str = "black";
const DEFAULT_TILE_SET: &str = YELLOW_TILE_SET;

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
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(TILE_STYLE_OPTION)
                    .description("Tile style")
                    .kind(ApplicationCommandOptionType::String)
                    .add_string_choice("Yellow", YELLOW_TILE_SET)
                    .add_string_choice("Red", RED_TILE_SET)
                    .add_string_choice("Black", BLACK_TILE_SET)
                    .required(false)
            });
    }

    async fn handle(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
        chombot: &Chombot,
    ) {
        let hand = command
            .data
            .options
            .iter()
            .find(|option| option.name == HAND_OPTION)
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap();
        let tile_set_option = command
            .data
            .options
            .iter()
            .find(|option| option.name == TILE_STYLE_OPTION)
            .cloned();
        let tile_set = if let Some(tile_set_option_value) = tile_set_option {
            tile_set_option_value
                .value
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        } else {
            DEFAULT_TILE_SET.to_owned()
        };
        let render_tile_set = match tile_set.as_str() {
            YELLOW_TILE_SET => TileStyle::Yellow,
            RED_TILE_SET => TileStyle::Red,
            BLACK_TILE_SET => TileStyle::Black,
            _ => unreachable!(),
        };

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }

        let image = chombot.render_hand(hand, render_tile_set).await.unwrap();
        let mut buf = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut buf, image::ImageOutputFormat::Png)
            .expect("Unable to write");

        let files: Vec<AttachmentType> = vec![(buf.as_slice(), "hand.png").into()];
        let image_message = command
            .channel_id
            .send_files(&ctx.http, files, |m| m)
            .await
            .unwrap();
        let link = image_message.link_ensured(&ctx.http).await;

        if let Err(why) = command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content(format!("Rendered hand: `{}`: {}", hand, link))
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
}
