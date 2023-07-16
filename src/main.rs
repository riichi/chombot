use anyhow::Error;
use clap::Parser;
use log::{error, info, LevelFilter};
use poise::serenity_prelude::{ChannelId, Context as SerenityContext, GatewayIntents};
use poise::{BoxFuture, Command, Context, Event, Framework, FrameworkContext, FrameworkOptions};

use crate::args::Arguments;
use crate::chombot::Chombot;
use crate::data_watcher::DataWatcher;
use crate::kcc3::data_types::{Chombo, DiscordId, Player, PlayerId};
use crate::kcc3::{Kcc3Client, Kcc3ClientResult};
use crate::ranking_watcher::notifier::ChannelMessageNotifier;
use crate::ranking_watcher::usma::get_ranking;
use crate::slash_commands::chombo::chombo;
use crate::slash_commands::hand::hand;
use crate::slash_commands::pasta::pasta;
use crate::slash_commands::score::score;
use crate::tournaments_watcher::ema::get_rcr_tournaments;
use crate::tournaments_watcher::notifier::TournamentsChannelMessageNotifier;

mod args;
mod chombot;
mod data;
mod data_watcher;
mod discord_utils;
mod kcc3;
mod ranking_watcher;
mod scraping_utils;
mod slash_commands;
mod tournaments_watcher;

const AT_EVERYONE_REACTIONS: [&str; 2] = ["Ichiangry", "Mikiknife"];

pub struct PoiseUserData {
    pub chombot: Chombot,
}

pub type PoiseContext<'a> = Context<'a, PoiseUserData, anyhow::Error>;

async fn start_ranking_watcher(args: &Arguments, ctx: SerenityContext) {
    if !args.feature_ranking_watcher {
        return;
    }
    let ranking_watcher_channel_id = args
        .ranking_watcher_channel_id
        .expect("Ranking watcher feature enabled but no channel ID provided");
    let notifier = ChannelMessageNotifier::new(
        ChannelId(ranking_watcher_channel_id),
        ctx,
        String::from("https://ranking.cvgo.re/ ranking update"),
    );
    tokio::spawn(async move {
        DataWatcher::new(notifier, get_ranking).run().await;
    });
}

async fn start_tournaments_watcher(args: &Arguments, ctx: SerenityContext) {
    if !args.feature_tournaments_watcher {
        return;
    }
    let tournemants_watcher_channel_id = args
        .tournaments_watcher_channel_id
        .expect("Tournaments watcher feature enabled but no channel ID provided");

    const MESSAGE_PREFIX: &str =
        "**TOURNAMENTS UPDATE** (http://mahjong-europe.org/ranking/Calendar.html)\n\n";
    let notifier = TournamentsChannelMessageNotifier::new(
        ChannelId(tournemants_watcher_channel_id),
        ctx,
        String::from(MESSAGE_PREFIX),
    );
    tokio::spawn(async move {
        DataWatcher::new(notifier, get_rcr_tournaments).run().await;
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
    event: &'a Event<'a>,
    _framework_ctx: FrameworkContext<'a, PoiseUserData, Error>,
    _user_data: &'a PoiseUserData,
) -> BoxFuture<'a, anyhow::Result<()>> {
    if let Event::Message { new_message } = event {
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
    let mut ret = vec![hand(), score()];
    if args.feature_kcc3 {
        ret.push(chombo());
    }
    if args.feature_pasta {
        ret.push(pasta());
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
    let chombot = chombot::Chombot::new(kcc3_client);

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: get_command_list(&args),
            event_handler,
            ..Default::default()
        })
        .token(&args.discord_token)
        .intents(GatewayIntents::non_privileged())
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                start_ranking_watcher(&args, ctx.clone()).await;
                start_tournaments_watcher(&args, ctx.clone()).await;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("{} is connected!", ready.user.name);
                Ok(PoiseUserData { chombot })
            })
        });

    if let Err(why) = framework.run().await {
        error!("Client error: {why:?}");
    }
}
