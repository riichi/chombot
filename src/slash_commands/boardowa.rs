use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use chrono::{Datelike, Local, Months, NaiveDate};
use reqwest::Client;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

use crate::boardowa::api::BoardowaAPIAdapter;
use crate::boardowa::models::TimeRange;
use crate::boardowa::ranking::{AvailabilityRange, RankingProvider};
use crate::slash_commands::utils::get_int_option;
use crate::slash_commands::{SlashCommand, SlashCommandResult};
use crate::Chombot;

const BOARDOWA_COMMAND: &str = "boardowa";
const DAY_OPTION: &str = "day";
const COUNT_OPTION: &str = "count";
const BEFORE_OPTION: &str = "before";
const AFTER_OPTION: &str = "after";
const DEFAULT_COUNT: i64 = 15;

pub struct BoardowaCommand {}

impl BoardowaCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SlashCommand for BoardowaCommand {
    fn get_name(&self) -> &'static str {
        BOARDOWA_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Get available tables at Boardowa")
            .create_option(|option| {
                option
                    .name(DAY_OPTION)
                    .description("Day of month you want to know about.")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(1)
                    .max_int_value(31)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(COUNT_OPTION)
                    .description(format!(
                        "How many possibilities? (at most) (default is {})",
                        DEFAULT_COUNT
                    ))
                    .kind(CommandOptionType::Integer)
                    .min_int_value(1)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name(AFTER_OPTION)
                    .description("The earliest hour")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(0)
                    .max_int_value(24)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name(BEFORE_OPTION)
                    .description("The latest hour")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(0)
                    .max_int_value(24)
                    .required(false)
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _chombot: &Chombot,
    ) -> SlashCommandResult {
        let date = get_query_date(command)?;
        let api = BoardowaAPIAdapter::new(Client::new());
        let opening = api.get_opening_info(&date).await?.range;
        let query_range = get_query_time_range(opening, command)?;
        let ranking = RankingProvider::new(api)
            .get_ranking(&date, &query_range)
            .await?;
        let option_count =
            *get_int_option(&command.data.options, COUNT_OPTION).unwrap_or(&DEFAULT_COUNT);
        let message = build_message(ranking, date, query_range, option_count.try_into()?);

        command
            .edit_original_interaction_response(&ctx.http, |response| response.content(message))
            .await?;

        Ok(())
    }
}

fn get_query_date(command: &ApplicationCommandInteraction) -> Result<NaiveDate> {
    let dom = u32::try_from(
        *get_int_option(&command.data.options, DAY_OPTION)
            .ok_or_else(|| anyhow!("Missing day of month"))?,
    )?;
    let now = Local::now().date_naive();
    let month_adjusted = if now.day() > dom {
        now.checked_add_months(Months::new(1))
            .ok_or_else(|| anyhow!("Internal error: date out of range"))?
    } else {
        now
    };
    month_adjusted.with_day(dom).ok_or_else(|| {
        anyhow!(
            "There is no day {:02} in {:02}/{}",
            dom,
            month_adjusted.month(),
            month_adjusted.year()
        )
    })
}

fn get_query_time_range(
    mut opening: TimeRange,
    command: &ApplicationCommandInteraction,
) -> Result<TimeRange> {
    if let Some(&from) = get_int_option(&command.data.options, AFTER_OPTION) {
        if i64::from(opening.from) < from {
            opening.from = from.try_into()?;
        }
    }
    if let Some(&to) = get_int_option(&command.data.options, BEFORE_OPTION) {
        if i64::from(opening.to) > to {
            opening.to = to.try_into()?;
        }
    }
    if opening.from >= opening.to {
        bail!(
            "`before` must be strictly before `after` (got: `{:02}-{:02}`)",
            opening.from,
            opening.to
        );
    }
    Ok(opening)
}

fn build_message(
    rank: Vec<AvailabilityRange>,
    date: NaiveDate,
    opening: TimeRange,
    n_tables: usize,
) -> String {
    let mut message = format!(
        "Consider following tables for {}:\n```\n",
        date.format("%Y-%m-%d")
    );
    message.push_str(
        &rank
            .into_iter()
            .take(n_tables)
            .map(|range| format_table(&range, &opening))
            .collect::<Vec<String>>()
            .join("\n"),
    );
    message.push_str("\n```");
    message
}

fn format_table(range: &AvailabilityRange, opening: &TimeRange) -> String {
    format!(
        "Table {} | {} ppl. | {}-{}",
        replace_leading_zeros(range.table_id.as_str()),
        range.capacity,
        format_time(opening.from.into(), range.start),
        format_time(opening.from.into(), range.start + (range.len as usize))
    )
}

fn replace_leading_zeros(id: &str) -> Cow<str> {
    if id.is_empty() || matches!(id.chars().next(), Some(c) if c != '0') {
        return id.into();
    }
    let idx = match id.char_indices().find(|(_, c)| c != &'0') {
        None => id.len() - 1,
        Some((idx, _)) => idx,
    };
    let mut ret = " ".repeat(idx);
    ret.push_str(&id[idx..]);
    ret.into()
}

fn format_time(start: usize, index: usize) -> String {
    let hour = index / 2 + start;
    let minutes = if index % 2 == 0 { 0 } else { 30 };
    format!("{:02}:{:02}", hour, minutes)
}
