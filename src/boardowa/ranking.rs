use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use chrono::NaiveDate;

use super::api::BoardowaAPIAdapter;
use super::models::{TableInfo as HourlyTableInfo, TimeRange};

struct TableInfo {
    pub value: String,
    pub capacity: u8,
    pub availability: Vec<bool>,
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
                            capacity: self.capacity,
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

#[derive(PartialEq, Eq)]
pub struct AvailabilityRange {
    pub table_id: String,
    pub start: usize,
    pub capacity: u8,
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

pub struct RankingProvider {
    api: BoardowaAPIAdapter,
}

impl RankingProvider {
    pub fn new(api: BoardowaAPIAdapter) -> Self {
        Self { api }
    }

    pub async fn get_ranking(
        &self,
        date: &NaiveDate,
        between: &TimeRange,
    ) -> Result<Vec<AvailabilityRange>> {
        let availabilities = self.get_table_availabilites(date, between).await?;
        let transposed = Self::transpose(availabilities)?;
        Ok(Self::rank_tables(transposed))
    }

    async fn get_table_availabilites(
        &self,
        date: &NaiveDate,
        between: &TimeRange,
    ) -> Result<Vec<Vec<HourlyTableInfo>>> {
        let mut ret = Vec::new();
        for t in between.from..between.to {
            ret.push(
                self.api
                    .get_tables_info(date, format!("{t:02}:01"), format!("{t:02}:30"))
                    .await?,
            );
            ret.push(
                self.api
                    .get_tables_info(date, format!("{t:02}:31"), format!("{:02}:00", t + 1))
                    .await?,
            );
        }
        Ok(ret)
    }

    fn transpose(availability: Vec<Vec<HourlyTableInfo>>) -> Result<Vec<TableInfo>> {
        let mut map = HashMap::new();
        if let Some(tables) = availability.first() {
            for table in tables {
                map.insert(
                    &table.value,
                    TableInfo {
                        value: table.value.clone(),
                        capacity: table.capacity,
                        availability: vec![],
                    },
                );
            }
        }
        for tables in &availability {
            if tables.len() != map.len() {
                bail!("Internal error: unexpected table")
            }
            for table in tables {
                map.get_mut(&table.value)
                    .ok_or_else(|| anyhow!("Missing table"))?
                    .availability
                    .push(table.available);
            }
        }
        Ok(map.into_values().collect())
    }

    fn rank_tables(tables: Vec<TableInfo>) -> Vec<AvailabilityRange> {
        let mut ret = Vec::new();
        for table in tables {
            ret.extend(table.availability_ranges())
        }
        ret.sort_unstable();
        ret
    }
}
