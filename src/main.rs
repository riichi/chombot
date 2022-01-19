use std::env;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            Interaction,
            InteractionResponseType,
        },
    },
    prelude::*,
};
use serenity::builder::CreateEmbed;
use serenity::model::channel::Message;
use serenity::model::interactions::application_command::{ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;
use slug::slugify;

use crate::chombot::Chombot;
use crate::kcc3::data_types::{Chombo, DiscordId, Player, PlayerId};
use crate::kcc3::Kcc3Client;

mod kcc3;
mod chombot;

const AT_EVERYONE_REACTIONS: [&'static str; 2] = ["Ichiangry", "Mikiknife"];

struct Handler {
    chombot: Chombot,
}

const DISCORD_MESSAGE_SIZE_LIMIT: usize = 2000;

const CHOMBO_COMMAND: &'static str = "chombo";
const CHOMBO_RANKING_SUBCOMMAND: &'static str = "ranking";
const CHOMBO_LIST_SUBCOMMAND: &'static str = "list";
const CHOMBO_ADD_SUBCOMMAND: &'static str = "add";

impl Handler {
    pub fn new(chombot: Chombot) -> Self {
        Self {
            chombot,
        }
    }

    async fn create_chombos_embed(&self) -> CreateEmbed {
        let chombos = self.chombot.create_chombo_ranking().await.unwrap();
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

    async fn create_chombos_list(&self) -> String {
        let chombos = self.chombot.get_chombo_list().await.unwrap();
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

    async fn react_to_chombo_command(&self, ctx: Context, command: ApplicationCommandInteraction) {
        let subcommand = command.data.options
            .iter()
            .find(|x| x.kind == ApplicationCommandOptionType::SubCommand)
            .unwrap();

        match subcommand.name.as_str() {
            CHOMBO_RANKING_SUBCOMMAND => self.react_to_chombo_ranking_subcommand(ctx, &command, subcommand).await,
            CHOMBO_LIST_SUBCOMMAND => self.react_to_chombo_list_subcommand(ctx, &command, subcommand).await,
            CHOMBO_ADD_SUBCOMMAND => self.react_to_chombo_add_subcommand(ctx, &command, subcommand).await,
            &_ => {}
        }
    }

    async fn react_to_chombo_ranking_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, subcommand: &ApplicationCommandInteractionDataOption) {
        let embed = self.create_chombos_embed().await;

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

    async fn react_to_chombo_list_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, subcommand: &ApplicationCommandInteractionDataOption) {
        let chombos = self.create_chombos_list().await;

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

    async fn react_to_chombo_add_subcommand(&self, ctx: Context, command: &ApplicationCommandInteraction, subcommand: &ApplicationCommandInteractionDataOption) {
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

        self.chombot.add_chombo_for_player(
            |player| player.discord_id.0 == user.id.to_string(),
            || Player::new_from_discord(PlayerId(slugify(&user.name)), user.name.clone(), DiscordId(user.id.to_string())),
            description,
        ).await.unwrap();

        let message_content = format!("Adding chombo for <@!{}>: *{}*", user.id, description);
        let embed = self.create_chombos_embed().await;

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

const CHOMBO_ADD_SUBCOMMAND_USER_OPTION: &'static str = "user";
const CHOMBO_ADD_SUBCOMMAND_DESCRIPTION_OPTION: &'static str = "description";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if !message.mention_everyone {
            return;
        }

        let emojis = message.guild_id.unwrap().emojis(&ctx.http).await.unwrap();
        let emojis: Vec<_> = emojis.into_iter()
            .filter(|x| AT_EVERYONE_REACTIONS.contains(&x.name.as_str()))
            .collect();

        for emoji in emojis {
            message.react(&ctx, emoji).await.unwrap();
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("{:?}", command.data.options);

            match command.data.name.as_str() {
                CHOMBO_COMMAND => self.react_to_chombo_command(ctx, command).await,
                _ => println!("Invalid command received"),
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
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
                        })
                })
        })
            .await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let kcc3_url = env::var("KCC3_URL")
        .expect("Expected KCC3 URL in the environment");
    let kcc3_token = env::var("KCC3_TOKEN")
        .expect("Expected KCC3 token in the environment");
    let kcc3client = kcc3::Kcc3Client::new(kcc3_url, &kcc3_token).unwrap();
    let chombot = chombot::Chombot::new(kcc3client);

    let mut client = Client::builder(token)
        .event_handler(Handler::new(chombot))
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
