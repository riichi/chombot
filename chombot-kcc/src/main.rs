#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

extern crate core;

use anyhow::Error;
use chombot_common::chombot::ChombotBase;
use chombot_common::data_watcher::DataWatcher;
use chombot_common::slash_commands::hand::hand;
use chombot_common::slash_commands::score::score;
use chombot_common::{start_tournaments_watcher, ChombotPoiseUserData};
use clap::Parser;
use log::{error, info, LevelFilter};
use poise::serenity_prelude::{
    ChannelId, ClientBuilder, Context as SerenityContext, FullEvent, GatewayIntents,
};
use poise::{BoxFuture, Command, Context, Framework, FrameworkContext, FrameworkOptions};

use crate::args::Arguments;
use crate::chombot::Chombot;
use crate::kcc3::Kcc3ClientResult;
use chombot_common::ranking_watcher::notifier::ChannelMessageNotifier;
use chombot_common::ranking_watcher::usma::get_ranking;
use crate::slash_commands::chombo::chombo;
use crate::slash_commands::pasta::pasta;

mod args;
mod chombot;
mod kcc3;
mod slash_commands;

const AT_EVERYONE_REACTIONS: [&str; 2] = ["Ichiangry", "Mikiknife"];

pub struct PoiseUserData {
    pub chombot: ChombotBase,
    pub kcc_chombot: Chombot,
}

impl ChombotPoiseUserData for PoiseUserData {
    fn chombot(&self) -> &ChombotBase {
        &self.chombot
    }
}

pub type PoiseContext<'a> = Context<'a, PoiseUserData, anyhow::Error>;

fn start_ranking_watcher(ranking_watcher_channel_id: Option<u64>, ctx: SerenityContext) {
    let ranking_watcher_channel_id = ranking_watcher_channel_id
        .expect("Ranking watcher feature enabled but no channel ID provided");
    let notifier = ChannelMessageNotifier::new(
        ChannelId::new(ranking_watcher_channel_id),
        String::from("https://ranking.cvgo.re/ ranking update"),
    );
    tokio::spawn(async move {
        DataWatcher::new(notifier, get_ranking).run(&ctx).await;
    });
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

fn event_handler<'a>(
    ctx: &'a SerenityContext,
    event: &'a FullEvent,
    _framework_ctx: FrameworkContext<'a, PoiseUserData, Error>,
    _user_data: &'a PoiseUserData,
) -> BoxFuture<'a, anyhow::Result<()>> {
    if let FullEvent::Message { new_message } = event {
        return Box::pin(async move {
            if !new_message.mention_everyone {
                return Ok(());
            }

            let emojis = new_message
                .guild_id
                .unwrap()
                .emojis(&ctx.http)
                .await
                .unwrap();
            let emojis: Vec<_> = emojis
                .into_iter()
                .filter(|x| AT_EVERYONE_REACTIONS.contains(&x.name.as_str()))
                .collect();

            for emoji in emojis {
                new_message.react(&ctx, emoji).await.unwrap();
            }

            Ok(())
        });
    }
    Box::pin(async { Ok(()) })
}

fn get_command_list(args: &Arguments) -> Vec<Command<PoiseUserData, Error>> {
    let mut ret: Vec<Command<PoiseUserData, Error>> = vec![hand(), score()];
    if args.feature_kcc3 {
        ret.push(chombo());
    }
    if args.feature_pasta {
        ret.push(pasta());
    }
    if args.feature_fancy_text {
        ret.push(slash_commands::fancy_text::fancy_text());
    }

    ret
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("chombot", LevelFilter::Info)
        .init();

    let args = Arguments::parse();
    let kcc3_client = get_kcc3_client(&args).unwrap();
    let chombot = ChombotBase::new();
    let kcc_chombot = Chombot::new(kcc3_client);

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: get_command_list(&args),
            event_handler,
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                if args.feature_ranking_watcher {
                    start_ranking_watcher(args.ranking_watcher_channel_id, ctx.clone());
                }
                if args.feature_tournaments_watcher {
                    let tournaments_watcher_channel_id =
                        ChannelId::from(args.tournaments_watcher_channel_id.expect(
                            "Tournaments watcher feature enabled but no channel ID provided",
                        ));
                    start_tournaments_watcher(tournaments_watcher_channel_id, ctx.clone());
                }
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("{} is connected!", ready.user.name);
                Ok(PoiseUserData {
                    chombot,
                    kcc_chombot,
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(&args.discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("Could not create client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
