use std::time::Duration;

use anyhow::Result;
use itertools::Itertools;

use crate::tournaments_watcher::ema::{TournamentEntry, TournamentStatus, TournamentStatuses};
use crate::tournaments_watcher::notifier::{TEST_MESSAGE_PREFIX, post_tournaments_update};
use crate::{ChombotPoiseContext, ChombotPoiseUserData};

const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Post a test TOURNAMENTS UPDATE with upcoming tournaments here.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn tournament_update_test<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "How many upcoming tournaments to list (default 3)"]
    #[min = 1]
    #[max = 20]
    count: Option<u8>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let Some(tournaments) = ctx
        .data()
        .chombot()
        .tournaments()
        .get_within(FETCH_TIMEOUT)
        .await
    else {
        ctx.say("Couldn't fetch the EMA calendar.").await?;
        return Ok(());
    };
    let statuses = upcoming_as_new(&tournaments.0, count.unwrap_or(3).into());
    if statuses.0.is_empty() {
        ctx.say("There are no upcoming tournaments on the EMA calendar.")
            .await?;
        return Ok(());
    }

    let all_reactions_added = post_tournaments_update(
        ctx.serenity_context(),
        ctx.data().chombot().tournament_updates(),
        ctx.channel_id(),
        TEST_MESSAGE_PREFIX,
        &statuses,
        true,
    )
    .await?;
    ctx.say(if all_reactions_added {
        "Posted. Click a number to create that tournament's channel."
    } else {
        "Posted, but I couldn't add the reactions: I need the Add Reactions and Read Message \
         History permissions here."
    })
    .await?;
    Ok(())
}

fn upcoming_as_new(tournaments: &[TournamentEntry], count: usize) -> TournamentStatuses {
    TournamentStatuses(
        tournaments
            .iter()
            .filter(|entry| entry.results_status.is_empty())
            .unique_by(|entry| (&entry.name, &entry.place))
            .take(count)
            .cloned()
            .map(TournamentStatus::New)
            .collect(),
    )
}
