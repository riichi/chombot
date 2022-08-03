use std::error::Error;

use async_trait::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};

use crate::slash_commands::chombo::ChomboCommand;
use crate::slash_commands::hand::HandCommand;
use crate::slash_commands::pasta::PastaCommand;
use crate::Chombot;

mod chombo;
mod hand;
mod pasta;
mod utils;

pub type SlashCommandResult = Result<(), Box<dyn Error>>;

#[async_trait]
pub trait SlashCommand: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn add_application_command(&self, command: &mut CreateApplicationCommand);
    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        chombot: &Chombot,
    ) -> SlashCommandResult;
}

pub struct SlashCommands {
    commands: Vec<Box<dyn SlashCommand>>,
}

impl SlashCommands {
    pub fn new() -> Self {
        Self {
            commands: Self::get_slash_commands(),
        }
    }

    fn get_slash_commands() -> Vec<Box<dyn SlashCommand>> {
        vec![
            Box::new(ChomboCommand::new()),
            Box::new(HandCommand::new()),
            Box::new(PastaCommand::new()),
        ]
    }

    pub fn register_commands(&self, commands: &mut CreateApplicationCommands) {
        for slash_command in &self.commands {
            commands.create_application_command(|command| {
                slash_command.add_application_command(command);
                command.name(slash_command.get_name())
            });
        }
    }

    fn get_command(&self, command_name: &str) -> Option<&dyn SlashCommand> {
        self.commands
            .iter()
            .find(|command| command.get_name() == command_name)
            .map(Box::as_ref)
    }

    async fn set_error_message(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        error_message: &str,
    ) {
        let error_response_result = command
            .edit_original_interaction_response(&ctx.http, |data| data.content(error_message))
            .await;

        if let Err(err) = error_response_result {
            println!("Could not set error response: {:?}", err);
        }
    }

    async fn handle_slash_command(
        slash_command: &dyn SlashCommand,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        chombot: &Chombot,
        requested_command_name: &str,
    ) -> Result<(), String> {
        if let Err(e) = slash_command.handle(ctx, command, chombot).await {
            println!(
                "Handler error for command {}: {:?}",
                requested_command_name, e
            );
            Err(format!("Could not generate response:\n```\n{}\n```", e))
        } else {
            Ok(())
        }
    }

    pub async fn handle(&self, ctx: Context, interaction: Interaction, chombot: &Chombot) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let deferred_result = command
                .create_interaction_response(&ctx.http, |response| {
                    response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                })
                .await;
            if let Err(e) = deferred_result {
                println!("Could not create deferred response: {:?}", e);
                return;
            }

            let requested_command_name = command.data.name.as_str();

            if let Some(slash_command) = self.get_command(requested_command_name) {
                if let Err(msg) = Self::handle_slash_command(
                    slash_command,
                    &ctx,
                    &command,
                    chombot,
                    requested_command_name,
                )
                .await
                {
                    Self::set_error_message(&ctx, &command, msg.as_str()).await;
                }
            } else {
                println!("Invalid command received: {}", requested_command_name);
            }
        }
    }
}
