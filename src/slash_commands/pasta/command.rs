use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

use crate::data::DISCORD_MESSAGE_SIZE_LIMIT;
use crate::slash_commands::utils::get_string_option;
use crate::slash_commands::{SlashCommand, SlashCommandResult};
use crate::Chombot;

const PASTA_COMMAND: &str = "pasta";
const PASTA_OPTION: &str = "pasta";

const JGAMESCON_PASTA_OPTION: &str = "jgamescon";
const JGAMESCON_PASTA: &str = include_str!("jgamescon.txt");
const TANJALO_PASTA_OPTION: &str = "tanjalo";
const TANJALO_PASTA: &str = include_str!("tanjalo.txt");
const STOWARZYSZENIE_PASTA_OPTION: &str = "stowarzyszenie";
const STOWARZYSZENIE_PASTA: &str = include_str!("stowarzyszenie.txt");

pub struct PastaCommand;

impl PastaCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SlashCommand for PastaCommand {
    fn get_name(&self) -> &'static str {
        PASTA_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Paste a pasta 🍝")
            .create_option(|option| {
                option
                    .name(PASTA_OPTION)
                    .description("Copypasta to output")
                    .kind(CommandOptionType::String)
                    .add_string_choice("O jgamesach na conach", JGAMESCON_PASTA_OPTION)
                    .add_string_choice("Expert 1 do wora a wór do jeziora", TANJALO_PASTA_OPTION)
                    .add_string_choice(
                        "Tylko nie zakładajcie mu stowarzyszenia",
                        STOWARZYSZENIE_PASTA_OPTION,
                    )
                    .required(true)
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _chombot: &Chombot,
    ) -> SlashCommandResult {
        let pasta_option = get_string_option(&command.data.options, PASTA_OPTION)
            .expect("Pasta option not provided");
        let pasta = match pasta_option {
            JGAMESCON_PASTA_OPTION => Ok(JGAMESCON_PASTA),
            TANJALO_PASTA_OPTION => Ok(TANJALO_PASTA),
            STOWARZYSZENIE_PASTA_OPTION => Ok(STOWARZYSZENIE_PASTA),
            _ => Err(format!("Invalid pasta: {}", pasta_option)),
        }?;
        let pasta_content = format!("{}\n||#pasta||", pasta.trim());

        let mut first = true;

        let mut message = String::new();
        for line in pasta_content.lines() {
            if message.len() + line.len() + "\n".len() > DISCORD_MESSAGE_SIZE_LIMIT {
                self.send_pasta_slice(ctx, command, &message, &mut first)
                    .await?;
                message.clear();
            }

            message.push_str(line);
            message.push('\n');
        }
        if !message.is_empty() {
            self.send_pasta_slice(ctx, command, &message, &mut first)
                .await?;
        }

        Ok(())
    }
}

impl PastaCommand {
    async fn send_pasta_slice(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        message: &str,
        first: &mut bool,
    ) -> SlashCommandResult {
        if *first {
            *first = false;
            command
                .edit_original_interaction_response(&ctx.http, |response| response.content(message))
                .await?;
        } else {
            command
                .channel_id
                .send_message(&ctx.http, |m| m.content(message))
                .await?;
        }

        Ok(())
    }
}
