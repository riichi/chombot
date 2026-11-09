use std::time::Duration;

use anyhow::{Result, anyhow};
use itertools::Itertools;
use poise::CreateReply;
use poise::serenity_prelude::{AutocompleteChoice, ChannelId, CreateAutocompleteResponse, GuildId};

use crate::channel_creator::{
    CHANNEL_NAME_MAX_LENGTH, ChannelCreation, ChannelName, Placement, TextChannelRequest,
    create_text_channel, decide_placement, truncate_chars,
};
use crate::tournament_channel::{
    polish_city, polish_date, tournament_channel_name, tournament_topic,
};
use crate::tournaments_watcher::ema::{TournamentEntry, Tournaments};
use crate::{ChombotPoiseContext, ChombotPoiseUserData};

const AUTOCOMPLETE_MAX_CHOICES: usize = 25;
/// Discord waits 3 seconds for autocomplete suggestions.
const AUTOCOMPLETE_TIMEOUT: Duration = Duration::from_millis(2500);
const TOURNAMENT_LOOKUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Create a text channel, e.g. for an upcoming EMA tournament.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_CHANNELS"
)]
pub async fn create_channel<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "Channel name; start typing to pick an EMA tournament"]
    #[autocomplete = "autocomplete_tournament"]
    #[max_length = 100]
    name: String,
    #[description = "Category to create the channel in (default: this channel's category)"]
    #[channel_types("Category")]
    category: Option<ChannelId>,
    #[description = "Channel topic (defaults to the tournament's place, date and website)"]
    #[max_length = 1024]
    topic: Option<String>,
) -> Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow!("Guild ID is None"))?;
    let Some(plain_name) = ChannelName::new(&name) else {
        ctx.send(
            CreateReply::default()
                .content("That name doesn't contain any characters usable in a channel name.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };
    let placement = match placement(ctx, guild_id, category).await? {
        Ok(placement) => placement,
        Err(reason) => {
            ctx.send(CreateReply::default().content(reason).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    ctx.defer().await?;
    let tournament = ctx
        .data()
        .chombot()
        .tournaments()
        .get_within(TOURNAMENT_LOOKUP_TIMEOUT)
        .await
        .and_then(|tournaments| find_tournament(&tournaments, &name, &plain_name).cloned());
    let creation = create_text_channel(
        ctx.serenity_context(),
        guild_id,
        TextChannelRequest {
            name: tournament
                .as_ref()
                .and_then(tournament_channel_name)
                .unwrap_or(plain_name),
            placement,
            topic: topic.or_else(|| tournament.as_ref().map(tournament_topic)),
            audit_log_reason: &format!("Requested by {} via /create_channel", ctx.author().name),
        },
    )
    .await?;

    let reply = match creation {
        ChannelCreation::Created(channel) => format!("Created <#{}>.", channel.id),
        ChannelCreation::AlreadyExists(channel) => format!("<#{}> already exists.", channel.id),
    };
    ctx.say(reply).await?;
    Ok(())
}

async fn placement<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    guild_id: GuildId,
    requested: Option<ChannelId>,
) -> Result<Result<Placement, String>> {
    let author = ctx
        .author_member()
        .await
        .ok_or_else(|| anyhow!("Could not get the invoking member"))?
        .into_owned();
    let bot = guild_id
        .member(ctx.serenity_context(), ctx.framework().bot_id())
        .await?;
    let current_channel = match requested {
        Some(_) => None,
        None => ctx.guild_channel().await,
    };

    let placement = ctx
        .guild()
        .map(|guild| decide_placement(&guild, &author, &bot, requested, current_channel.as_ref()));
    placement.ok_or_else(|| anyhow!("Guild {guild_id} is not cached"))
}

async fn autocomplete_tournament<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    partial: &str,
) -> CreateAutocompleteResponse {
    let Some(tournaments) = ctx
        .data()
        .chombot()
        .tournaments()
        .get_within(AUTOCOMPLETE_TIMEOUT)
        .await
    else {
        return CreateAutocompleteResponse::new();
    };

    let recent = ctx
        .data()
        .chombot()
        .tournament_updates()
        .recent_tournaments();
    let choices = matching_tournaments(&tournaments, &recent, partial)
        .into_iter()
        .map(|entry| {
            let label = choice_label(entry);
            AutocompleteChoice::new(label.clone(), label)
        })
        .collect();
    CreateAutocompleteResponse::new().set_choices(choices)
}

/// Tournaments from the most recent updates (`recent`) first, then upcoming
/// ones, then the rest.
fn matching_tournaments<'a>(
    tournaments: &'a Tournaments,
    recent: &[String],
    partial: &str,
) -> Vec<&'a TournamentEntry> {
    let partial = partial.trim().to_lowercase();
    let mut matching: Vec<&TournamentEntry> = tournaments
        .0
        .iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(&partial)
                || entry.place.to_lowercase().contains(&partial)
        })
        .unique_by(|entry| (&entry.name, &entry.place, &entry.date))
        .collect();
    matching.sort_by_key(|entry| {
        let recent_rank = recent.iter().position(|name| *name == entry.name);
        (
            recent_rank.unwrap_or(usize::MAX),
            !entry.results_status.is_empty(),
        )
    });
    matching.truncate(AUTOCOMPLETE_MAX_CHOICES);
    matching
}

/// E.g. `Hatsumi Taikai 2026 (14-15 lutego 2026, Kraków)`. Picking a
/// suggestion submits its label, which tells namesakes apart.
fn choice_label(entry: &TournamentEntry) -> String {
    let details = [polish_date(&entry.date), polish_city(&entry.place)]
        .into_iter()
        .filter(|detail| !detail.is_empty())
        .join(", ");
    let label = if details.is_empty() {
        entry.name.clone()
    } else {
        format!("{} ({details})", entry.name)
    };
    truncate_chars(&label, CHANNEL_NAME_MAX_LENGTH)
}

/// The tournament with the label `name`, or else named `name`, preferring
/// one without results yet.
fn find_tournament<'a>(
    tournaments: &'a Tournaments,
    name: &str,
    channel_name: &ChannelName,
) -> Option<&'a TournamentEntry> {
    let by_label = tournaments
        .0
        .iter()
        .find(|entry| choice_label(entry) == name.trim());
    let mut by_name = tournaments
        .0
        .iter()
        .filter(|entry| ChannelName::new(&entry.name).as_ref() == Some(channel_name));

    by_label.or_else(|| {
        by_name
            .clone()
            .find(|entry| entry.results_status.is_empty())
            .or_else(|| by_name.next())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, date: &str, place: &str, results: &str) -> TournamentEntry {
        TournamentEntry {
            name: name.to_owned(),
            url: String::new(),
            rules: "Riichi".to_owned(),
            date: date.to_owned(),
            place: place.to_owned(),
            approval_status: "OK".to_owned(),
            registration_start: String::new(),
            results_status: results.to_owned(),
        }
    }

    fn calendar() -> Tournaments {
        Tournaments(vec![
            entry(
                "Pumpkin Tiles 2026",
                "10-11 October 2026",
                "Tbilisi",
                "EMA waiting for results !",
            ),
            entry("Pumpkin Tiles 2026", "31 October 2026", "Yerevan", ""),
            entry("Warsaw Riichi Cup", "5 December 2026", "Warsaw", ""),
            entry("Warsaw Riichi Cup", "5 December 2026", "Warsaw", ""),
            entry("Hatsumi Taikai 2027", "13-14 February 2027", "Krakow", ""),
        ])
    }

    #[test]
    fn suggests_recent_then_upcoming_tournaments() {
        let calendar = calendar();
        let suggest = |recent: &[&str], partial: &str| -> Vec<String> {
            let recent: Vec<String> = recent.iter().map(|&name| name.to_owned()).collect();
            matching_tournaments(&calendar, &recent, partial)
                .into_iter()
                .map(choice_label)
                .collect()
        };

        assert_eq!(
            suggest(&["Hatsumi Taikai 2027"], ""),
            vec![
                "Hatsumi Taikai 2027 (13-14 lutego 2027, Kraków)",
                "Pumpkin Tiles 2026 (31 października 2026, Erywań)",
                "Warsaw Riichi Cup (5 grudnia 2026, Warszawa)",
                "Pumpkin Tiles 2026 (10-11 października 2026, Tbilisi)",
            ]
        );
        assert_eq!(
            suggest(&[], "WARSAW"),
            vec!["Warsaw Riichi Cup (5 grudnia 2026, Warszawa)"]
        );
    }

    #[test]
    fn finds_picked_or_typed_tournament() {
        let calendar = calendar();
        let find = |name: &str| {
            find_tournament(&calendar, name, &ChannelName::new(name).unwrap())
                .map(|entry| entry.place.as_str())
        };

        assert_eq!(
            find("Pumpkin Tiles 2026 (10-11 października 2026, Tbilisi)"),
            Some("Tbilisi")
        );
        assert_eq!(find("pumpkin tiles 2026"), Some("Yerevan"));
        assert_eq!(find("Something else"), None);
    }
}
