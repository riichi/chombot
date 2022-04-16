use std::env;

use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::{
    async_trait,
    model::{gateway::Ready, id::GuildId, interactions::Interaction},
    prelude::*,
};

use crate::chombot::Chombot;
use crate::kcc3::data_types::{Chombo, DiscordId, Player, PlayerId};
use crate::kcc3::Kcc3Client;
use crate::ranking_watcher::notifier::ChannelMessageNotifier;
use crate::ranking_watcher::usma::get_ranking;
use crate::ranking_watcher::RankingWatcher;
use crate::slash_commands::SlashCommands;

mod chombot;
mod kcc3;
mod ranking_watcher;
mod slash_commands;

const AT_EVERYONE_REACTIONS: [&str; 2] = ["Ichiangry", "Mikiknife"];

struct Handler {
    chombot: Chombot,
    slash_commands: SlashCommands,
}

impl Handler {
    pub fn new(chombot: Chombot) -> Self {
        Self {
            chombot,
            slash_commands: SlashCommands::new(),
        }
    }
}

async fn start_ranking_watcher(ctx: Context) {
    let ranking_watcher_channel_id = ChannelId(
        env::var("RANKING_WATCHER_CHANNEL_ID")
            .expect("Expected RANKING_WATCHER_CHANNEL_ID in environment")
            .parse()
            .expect("RANKING_WATCHER_CHANNEL_ID must be an integer"),
    );
    let notifier = ChannelMessageNotifier::new(
        ranking_watcher_channel_id,
        ctx,
        String::from("https://ranking.cvgo.re/ ranking update"),
    );
    tokio::spawn(async move {
        RankingWatcher::new(notifier, get_ranking).run().await;
    });
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if !message.mention_everyone {
            return;
        }

        let emojis = message.guild_id.unwrap().emojis(&ctx.http).await.unwrap();
        let emojis: Vec<_> = emojis
            .into_iter()
            .filter(|x| AT_EVERYONE_REACTIONS.contains(&x.name.as_str()))
            .collect();

        for emoji in emojis {
            message.react(&ctx, emoji).await.unwrap();
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.slash_commands
            .handle(ctx, interaction, &self.chombot)
            .await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            self.slash_commands.register_commands(commands);
            commands
        })
        .await
        .unwrap();

        start_ranking_watcher(ctx.clone()).await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let kcc3_url = env::var("KCC3_URL").expect("Expected KCC3 URL in the environment");
    let kcc3_token = env::var("KCC3_TOKEN").expect("Expected KCC3 token in the environment");
    let kcc3client = kcc3::Kcc3Client::new(kcc3_url, &kcc3_token).unwrap();
    let chombot = chombot::Chombot::new(kcc3client);

    let handler = Handler::new(chombot);
    let mut client = Client::builder(token)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
