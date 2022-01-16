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
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::chombot::Chombot;
use crate::kcc3::Kcc3Client;

mod kcc3;
mod chombot;

const AT_EVERYONE_REACTIONS: [&'static str; 2] = ["Ichiangry", "Mikiknife"];

struct Handler {
    chombot: Chombot,
}

impl Handler {
    pub fn new(chombot: Chombot) -> Self {
        Self {
            chombot,
        }
    }

    async fn create_chombos_embed(&self) -> CreateEmbed {
        let chombos = self.chombot.list_chombos_by_count().await.unwrap();
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

    async fn react_to_chombos_command(&self, ctx: Context, command: ApplicationCommandInteraction) {
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
}

const CHOMBOS_COMMAND: &'static str = "chombo";

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
            match command.data.name.as_str() {
                CHOMBOS_COMMAND => self.react_to_chombos_command(ctx, command).await,
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
                    command.name(CHOMBOS_COMMAND).description("List all chombos")
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
