use clap::Parser;
use log::{info, LevelFilter};
use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::*;

use crate::args::Arguments;
use crate::chombot::Chombot;
use crate::kcc3::data_types::{Chombo, DiscordId, Player, PlayerId};
use crate::kcc3::{Kcc3Client, Kcc3ClientResult};
use crate::ranking_watcher::notifier::ChannelMessageNotifier;
use crate::ranking_watcher::usma::get_ranking;
use crate::ranking_watcher::RankingWatcher;
use crate::slash_commands::SlashCommands;

mod args;
mod chombot;
mod data;
mod kcc3;
mod ranking_watcher;
mod slash_commands;

const AT_EVERYONE_REACTIONS: [&str; 2] = ["Ichiangry", "Mikiknife"];

struct Handler {
    chombot: Chombot,
    slash_commands: SlashCommands,
    args: Arguments,
}

struct HandlerState {
    ranking_watcher_started: bool,
}

impl TypeMapKey for HandlerState {
    type Value = Self;
}

impl Handler {
    pub fn new(chombot: Chombot, args: Arguments) -> Self {
        Self {
            chombot,
            slash_commands: SlashCommands::new(&args),
            args,
        }
    }

    async fn update_state<F, R>(ctx: &Context, callback: F) -> R
    where
        F: FnOnce(&mut HandlerState) -> R,
    {
        let mut data = ctx.data.write().await;
        match data.get_mut::<HandlerState>() {
            None => {
                let mut state = HandlerState {
                    ranking_watcher_started: false,
                };
                let ret = callback(&mut state);
                data.insert::<HandlerState>(state);
                ret
            }
            Some(state) => callback(state),
        }
    }

    async fn start_ranking_watcher(&self, ctx: Context) {
        if !self.args.feature_ranking_watcher {
            return;
        }
        let ranking_watcher_started = Self::update_state(&ctx, |state| {
            let ret = state.ranking_watcher_started;
            state.ranking_watcher_started = true;
            ret
        })
        .await;
        if ranking_watcher_started {
            return;
        }
        let ranking_watcher_channel_id = self
            .args
            .ranking_watcher_channel_id
            .expect("Ranking watcher feature enabled but no channel ID provided");
        let notifier = ChannelMessageNotifier::new(
            ChannelId(ranking_watcher_channel_id),
            ctx,
            String::from("https://ranking.cvgo.re/ ranking update"),
        );
        tokio::spawn(async move {
            RankingWatcher::new(notifier, get_ranking).run().await;
        });
    }
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
        self.start_ranking_watcher(ctx.clone()).await;

        info!("{} is connected!", ready.user.name);

        GuildId::set_application_commands(&GuildId(self.args.guild_id), &ctx.http, |commands| {
            self.slash_commands.register_commands(commands);
            commands
        })
        .await
        .unwrap();
    }
}

fn get_kcc3_client(args: &Arguments) -> Kcc3ClientResult<Option<kcc3::Kcc3Client>> {
    if !args.feature_kcc3 {
        return Ok(None);
    }
    let url = args
        .kcc3_url
        .as_ref()
        .expect("KCC3 feature enabled but no URL provided");
    let token = args
        .kcc3_token
        .as_ref()
        .expect("KCC3 feature enabled but no token provided");
    Ok(Some(kcc3::Kcc3Client::new(url.clone(), token)?))
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("chombot", LevelFilter::Info)
        .init();

    let args = Arguments::parse();

    let kcc3_client = get_kcc3_client(&args).unwrap();
    let chombot = chombot::Chombot::new(kcc3_client);

    let discord_token = args.discord_token.clone();
    let application_id = args.application_id;
    let handler = Handler::new(chombot, args);
    let mut client = Client::builder(discord_token, GatewayIntents::non_privileged())
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        info!("Client error: {why:?}");
    }
}
