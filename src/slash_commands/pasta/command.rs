use async_trait::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

use crate::slash_commands::utils::get_string_option;
use crate::slash_commands::{SlashCommand, SlashCommandResult};
use crate::Chombot;

const PASTA_COMMAND: &str = "pasta";
const PASTA_OPTION: &str = "pasta";

const JGAMESCON_PASTA_OPTION: &str = "jgamescon";
const JGAMESCON_PASTA: &str = include_str!("jgamescon.txt");

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
            _ => Err(format!("Invalid pasta: {}", pasta_option)),
        }?;
        let pasta_content = format!("{}\n||#pasta||", pasta.trim());

        command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.content(pasta_content)
            })
            .await?;

        Ok(())
    }
}
