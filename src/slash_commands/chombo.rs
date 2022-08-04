use std::error::Error;

use async_trait::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::prelude::User;
use serenity::utils::Colour;
use slug::slugify;

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;
use crate::slash_commands::utils::{get_string_option, get_user_option};
use crate::slash_commands::{SlashCommand, SlashCommandResult};
use crate::{Chombo, Chombot, DiscordId, Player, PlayerId};

const CHOMBO_COMMAND: &str = "chombo";
const CHOMBO_RANKING_SUBCOMMAND: &str = "ranking";
const CHOMBO_LIST_SUBCOMMAND: &str = "list";
const CHOMBO_ADD_SUBCOMMAND: &str = "add";
const CHOMBO_ADD_SUBCOMMAND_USER_OPTION: &str = "user";
const CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION: &str = "description";

pub struct ChomboCommand;

impl ChomboCommand {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_list_subcommand(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _subcommand: &CommandDataOption,
        chombot: &Chombot,
    ) -> SlashCommandResult {
        let chombos = Self::create_chombos_list(chombot).await?;

        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response
                    .content(chombos)
                    .allowed_mentions(|mentions| mentions.empty_parse())
            })
            .await?;

        Ok(())
    }

    async fn create_chombos_list(chombot: &Chombot) -> Result<String, Box<dyn Error>> {
        let chombos = chombot.get_chombo_list().await?;
        let mut result = String::new();
        for (player, chombo) in &chombos {
            let entry = Self::format_chombo_entry(player, chombo);
            if result.len() + entry.len() <= DISCORD_MESSAGE_SIZE_LIMIT {
                result += &entry;
            } else {
                break;
            }
        }

        Ok(result)
    }

    fn format_chombo_entry(player: &Player, chombo: &Chombo) -> String {
        let comment = if chombo.comment.is_empty() {
            "".to_owned()
        } else {
            format!(": *{}*", chombo.comment)
        };
        let timestamp = chombo.timestamp.format("%A, %Y-%m-%d %H:%M");

        format!("<@!{}> at {}{}\n", player.discord_id, timestamp, comment)
    }

    async fn handle_ranking_subcommand(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _subcommand: &CommandDataOption,
        chombot: &Chombot,
    ) -> SlashCommandResult {
        let embed = Self::create_chombos_embed(chombot).await?;

        command
            .edit_original_interaction_response(&ctx.http, |response| response.add_embed(embed))
            .await?;

        Ok(())
    }

    async fn handle_add_subcommand(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        subcommand: &CommandDataOption,
        chombot: &Chombot,
    ) -> SlashCommandResult {
        let (user, _) = get_user_option(&subcommand.options, CHOMBO_ADD_SUBCOMMAND_USER_OPTION)
            .ok_or("Missing user")?;
        let description = get_string_option(
            &subcommand.options,
            CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION,
        )
        .ok_or("Missing description")?;

        chombot
            .add_chombo_for_player(
                |player| player.discord_id.0 == user.id.to_string(),
                || {
                    Player::new_from_discord(
                        PlayerId(slugify(&user.name)),
                        user.name.clone(),
                        DiscordId(user.id.to_string()),
                    )
                },
                description,
            )
            .await?;

        let message_content = Self::format_add_message(user, description);
        let embed = Self::create_chombos_embed(chombot).await?;

        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content(message_content).add_embed(embed)
            })
            .await?;

        Ok(())
    }

    fn format_add_message(user: &User, description: &str) -> String {
        format!("Adding chombo for <@!{}>: *{}*", user.id, description)
    }

    async fn create_chombos_embed(chombot: &Chombot) -> Result<CreateEmbed, Box<dyn Error>> {
        let chombos = chombot.create_chombo_ranking().await?;
        let chombos = chombos
            .into_iter()
            .map(|(player, num)| (player.short_name(), num, true));

        let mut embed = CreateEmbed::default();
        embed
            .title("**CHOMBO COUNTER**")
            .color(Colour::RED)
            .thumbnail("https://cdn.discordapp.com/attachments/591385176685281293/597292309792686090/1562356453777.png")
            .fields(chombos);

        Ok(embed)
    }
}

#[async_trait]
impl SlashCommand for ChomboCommand {
    fn get_name(&self) -> &'static str {
        CHOMBO_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .description("List all chombos")
            .create_option(|option| {
                option
                    .name(CHOMBO_RANKING_SUBCOMMAND)
                    .description("Display the chombo ranking")
                    .kind(CommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name(CHOMBO_LIST_SUBCOMMAND)
                    .description("List all chombos")
                    .kind(CommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name(CHOMBO_ADD_SUBCOMMAND)
                    .description("Add a chombo for a user")
                    .kind(CommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name(CHOMBO_ADD_SUBCOMMAND_USER_OPTION)
                            .description("User that made a chombo")
                            .kind(CommandOptionType::User)
                            .required(true)
                    })
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name(CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION)
                            .description("Chombo description")
                            .kind(CommandOptionType::String)
                            .required(true)
                    })
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        chombot: &Chombot,
    ) -> SlashCommandResult {
        let subcommand = command
            .data
            .options
            .iter()
            .find(|x| x.kind == CommandOptionType::SubCommand)
            .unwrap();

        match subcommand.name.as_str() {
            CHOMBO_RANKING_SUBCOMMAND => {
                self.handle_ranking_subcommand(ctx, command, subcommand, chombot)
                    .await?
            }
            CHOMBO_LIST_SUBCOMMAND => {
                self.handle_list_subcommand(ctx, command, subcommand, chombot)
                    .await?
            }
            CHOMBO_ADD_SUBCOMMAND => {
                self.handle_add_subcommand(ctx, command, subcommand, chombot)
                    .await?
            }
            &_ => {}
        }

        Ok(())
    }
}
