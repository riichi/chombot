use anyhow::Result;
use chrono::NaiveDate;
use reqwest::Client;

use super::models;

pub struct BoardowaAPIAdapter {
    client: Client,
}

impl BoardowaAPIAdapter {
    const API_TABLES_URL: &str = "https://retable.herokuapp.com/availability/tables/";
    const API_TIMES_URL: &str = "https://retable.herokuapp.com/availability/times/";
    const DATE_FORMAT: &str = "%Y-%m-%d";

    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get_opening_info(&self, at: &NaiveDate) -> Result<models::OpeningInfo> {
        Ok(self
            .client
            .get(Self::API_TIMES_URL)
            .query(&[("date", at.format(Self::DATE_FORMAT).to_string())])
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_tables_info(
        &self,
        at: &NaiveDate,
        from: String,
        to: String,
    ) -> Result<Vec<models::TableInfo>> {
        Ok(self
            .client
            .get(Self::API_TABLES_URL)
            .query(&[
                ("date", at.format(Self::DATE_FORMAT).to_string()),
                ("time", format!("{}-{}", from, to)),
            ])
            .send()
            .await?
            .json()
            .await?)
    }
}
