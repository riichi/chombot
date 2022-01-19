use async_trait::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::client::Context;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType};
use serenity::model::interactions::InteractionResponseType;
use serenity::utils::Colour;
use slug::slugify;

use crate::{Chombo, Chombot, DiscordId, Player, PlayerId};
use crate::slash_commands::SlashCommand;

const DISCORD_MESSAGE_SIZE_LIMIT: usize = 2000;

const CHOMBO_COMMAND: &'static str = "chombo";
const CHOMBO_RANKING_SUBCOMMAND: &'static str = "ranking";
const CHOMBO_LIST_SUBCOMMAND: &'static str = "list";
const CHOMBO_ADD_SUBCOMMAND: &'static str = "add";
const CHOMBO_ADD_SUBCOMMAND_USER_OPTION: &'static str = "user";
const CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION: &'static str = "description";

pub struct ChomboCommand;

impl ChomboCommand {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_chombos_embed(chombot: &Chombot) -> CreateEmbed {
        let chombos = chombot.create_chombo_ranking().await.unwrap();
        let chombos = chombos.into_iter()
            .map(|(player, num)| (player.short_name(), num, true));

        let mut embed = CreateEmbed::default();
        embed
            .title("**CHOMBO COUNTER**")
            .color(Colour::RED)
            .thumbnail("https://cdn.discordapp.com/attachments/591385176685281293/597292309792686090/1562356453777.png")
            .fields(chombos);
        embed
    }

    async fn create_chombos_list(chombot: &Chombot) -> String {
        let chombos = chombot.get_chombo_list().await.unwrap();
        let mut result = String::new();
        for (player, chombo) in &chombos {
            let entry = Self::format_chombo_entry(player, chombo);
            if result.len() + entry.len() <= DISCORD_MESSAGE_SIZE_LIMIT {
                result += &entry;
            } else {
                break;
            }
        }

        result
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

    async fn react_to_chombo_ranking_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, _subcommand: &ApplicationCommandInteractionDataOption, chombot: &Chombot) {
        let embed = Self::create_chombos_embed(chombot).await;

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.add_embed(embed))
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }

    async fn react_to_chombo_list_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, _subcommand: &ApplicationCommandInteractionDataOption, chombot: &Chombot) {
        let chombos = Self::create_chombos_list(chombot).await;

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(chombos)
                            .allowed_mentions(|mentions| mentions.empty_parse())
                    })
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }

    async fn react_to_chombo_add_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, subcommand: &ApplicationCommandInteractionDataOption, chombot: &Chombot) {
        let user_option = subcommand.options.iter()
            .find(|option| option.name == CHOMBO_ADD_SUBCOMMAND_USER_OPTION)
            .unwrap()
            .resolved
            .as_ref()
            .unwrap();
        let user = match user_option {
            ApplicationCommandInteractionDataOptionValue::User(user, _) => user,
            _ => panic!("Invalid option value")
        };

        let description = subcommand.options.iter()
            .find(|option| option.name == CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION)
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap();

        chombot.add_chombo_for_player(
            |player| player.discord_id.0 == user.id.to_string(),
            || Player::new_from_discord(PlayerId(slugify(&user.name)), user.name.clone(), DiscordId(user.id.to_string())),
            description,
        ).await.unwrap();

        let message_content = format!("Adding chombo for <@!{}>: *{}*", user.id, description);
        let embed = Self::create_chombos_embed(chombot).await;


        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message
                        .content(message_content)
                        .add_embed(embed))
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
}

#[async_trait]
impl SlashCommand for ChomboCommand {
    fn get_name(&self) -> &'static str {
        CHOMBO_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .name(CHOMBO_COMMAND)
            .description("List all chombos")
            .create_option(|option| {
                option
                    .name(CHOMBO_RANKING_SUBCOMMAND)
                    .description("Display the chombo ranking")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name(CHOMBO_LIST_SUBCOMMAND)
                    .description("List all chombos")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name(CHOMBO_ADD_SUBCOMMAND)
                    .description("Add a chombo for a user")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name(CHOMBO_ADD_SUBCOMMAND_USER_OPTION)
                            .description("User that made a chombo")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name(CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION)
                            .description("Chombo description")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            });
    }

    async fn handle(&self, ctx: Context, command: ApplicationCommandInteraction, chombot: &Chombot) {
        let subcommand = command.data.options
            .iter()
            .find(|x| x.kind == ApplicationCommandOptionType::SubCommand)
            .unwrap();

        match subcommand.name.as_str() {
            CHOMBO_RANKING_SUBCOMMAND => self.react_to_chombo_ranking_subcommand(ctx, &command, subcommand, chombot).await,
            CHOMBO_LIST_SUBCOMMAND => self.react_to_chombo_list_subcommand(ctx, &command, subcommand, chombot).await,
            CHOMBO_ADD_SUBCOMMAND => self.react_to_chombo_add_subcommand(ctx, &command, subcommand, chombot).await,
            &_ => {}
        }
    }
}
