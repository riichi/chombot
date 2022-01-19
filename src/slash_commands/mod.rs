use async_trait::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::Interaction;

use crate::Chombot;
use crate::slash_commands::chombo::ChomboCommand;

mod chombo;

#[async_trait]
pub trait SlashCommand: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn add_application_command(&self, command: &mut CreateApplicationCommand);
    async fn handle(&self, ctx: Context, command: ApplicationCommandInteraction, chombot: &Chombot);
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
            Box::new(ChomboCommand::new())
        ]
    }

    pub fn register_commands(&self, commands: &mut CreateApplicationCommands) {
        for slash_command in &self.commands {
            commands
                .create_application_command(|command| {
                    slash_command.add_application_command(command);
                    command.name(slash_command.get_name())
                });
        }
    }

    pub async fn handle(&self, ctx: Context, interaction: Interaction, chombot: &Chombot) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let requested_command_name = command.data.name.as_str();
            let command_option = self.commands.iter()
                .find(|command| command.get_name() == requested_command_name);

            if let Some(slash_command) = command_option {
                slash_command.handle(ctx, command, chombot).await;
            } else {
                println!("Invalid command received");
            }
        }
    }
}
