use std::cmp::{Eq, Ord, PartialEq};
use std::collections::{BinaryHeap, HashMap};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;

use async_trait::async_trait;
use chrono::{Datelike, Local, Months};
use reqwest::Client;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;

use crate::boardowa::{get_opening_info, get_tables_info, models::TableInfo as HourlyTableInfo};
use crate::slash_commands::utils::get_int_option;
use crate::slash_commands::{SlashCommand, SlashCommandResult};
use crate::Chombot;

const BOARDOWA_COMMAND: &str = "boardowa";
const DAY_OPTION: &str = "day";

pub struct BoardowaCommand {}

impl BoardowaCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct BoardowaError(String);

impl Display for BoardowaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for BoardowaError {}

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
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _chombot: &Chombot,
    ) -> SlashCommandResult {
        let dom = u32::try_from(
            *get_int_option(&command.data.options, DAY_OPTION).ok_or("Missing day of month")?,
        )?;
        let now = Local::now().date_naive();
        let month_adjusted = if now.day() > dom {
            now.checked_add_months(Months::new(1))
                .ok_or("Internal error: date out of range")?
        } else {
            now
        };
        let day_adjusted = month_adjusted.with_day(dom).ok_or(format!(
            "There is no day {:02} in {:02}/{}",
            dom,
            month_adjusted.month(),
            month_adjusted.year()
        ))?;
        let client = Client::new();
        let opening = get_opening_info(&client, day_adjusted).await?.range;
        let mut availability_vec = vec![];
        for t in opening.from..opening.to {
            availability_vec.push(
                get_tables_info(
                    &client,
                    day_adjusted,
                    format!("{t:02}:01"),
                    format!("{t:02}:30"),
                )
                .await?,
            );
            availability_vec.push(
                get_tables_info(
                    &client,
                    day_adjusted,
                    format!("{t:02}:31"),
                    format!("{:02}:00", t + 1),
                )
                .await?,
            );
        }
        let transposed = transpose(availability_vec)?;
        let rank: Vec<AvailabilityRange> = rank_tables(transposed);

        let format_time = |idx| {
            let hour: usize = idx / 2 + (opening.from as usize);
            let minutes = if idx % 2 == 0 { 0 } else { 30 };
            format!("{hour:02}:{minutes:02}")
        };

        let mut message = format!(
            "Consider following tables for {}:\n```\n",
            day_adjusted.format("%Y-%m-%d")
        );
        message.push_str(
            &rank
                .iter()
                .take(5)
                .map(|range| {
                    format!(
                        "Table {}: {}-{}",
                        range.table_id,
                        format_time(range.start),
                        format_time(range.start + (range.len as usize))
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        );
        message.push_str("\n```");

        command
            .edit_original_interaction_response(&ctx.http, |response| response.content(message))
            .await?;

        Ok(())
    }
}

struct TableInfo {
    pub value: String,
    pub availability: Vec<bool>,
}

#[derive(PartialEq, Eq)]
struct AvailabilityRange {
    pub table_id: String,
    pub start: usize,
    pub len: u8,
}

impl PartialOrd for AvailabilityRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.len != other.len {
            other.len.partial_cmp(&self.len)
        } else {
            other.start.partial_cmp(&self.start)
        }
    }
}

impl Ord for AvailabilityRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.len != other.len {
            other.len.cmp(&self.len)
        } else {
            other.start.cmp(&self.start)
        }
    }
}

impl TableInfo {
    fn availability_ranges(&self) -> Vec<AvailabilityRange> {
        let mut ret = vec![];
        let mut current_range = None;
        for (idx, av) in self.availability.iter().enumerate() {
            if !av {
                if let Some(r) = current_range {
                    ret.push(r);
                    current_range = None;
                }
            } else {
                match &mut current_range {
                    None => {
                        current_range = Some(AvailabilityRange {
                            table_id: self.value.clone(),
                            start: idx,
                            len: 1,
                        });
                    }
                    Some(r) => {
                        r.len += 1;
                    }
                }
            }
        }
        if let Some(r) = current_range {
            ret.push(r);
        }
        ret
    }
}

fn transpose(availability: Vec<Vec<HourlyTableInfo>>) -> Result<Vec<TableInfo>, Box<dyn Error>> {
    let mut map = HashMap::new();
    if let Some(tables) = availability.first() {
        for table in tables {
            map.insert(
                &table.value,
                TableInfo {
                    value: table.value.clone(),
                    availability: vec![],
                },
            );
        }
    }
    for tables in &availability {
        if tables.len() != map.len() {
            return Err(Box::new(BoardowaError(String::from(
                "Internal error: unexpected table",
            ))));
        }
        for table in tables {
            map.get_mut(&table.value)
                .ok_or("Missing table")?
                .availability
                .push(table.available);
        }
    }
    Ok(map.into_values().collect())
}

fn rank_tables(tables: Vec<TableInfo>) -> Vec<AvailabilityRange> {
    let mut heap = BinaryHeap::new();
    for table in tables {
        for range in table.availability_ranges() {
            heap.push(range);
        }
    }
    heap.into_sorted_vec()
}
