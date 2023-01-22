use chrono::NaiveDate;
use reqwest::Client;
use std::error::Error;

pub mod models;

const API_TABLES_URL: &str = "https://retable.herokuapp.com/availability/tables/";
const API_TIMES_URL: &str = "https://retable.herokuapp.com/availability/times/";
const DATE_FORMAT: &str = "%Y-%m-%d";

pub async fn get_opening_info(
    client: &Client,
    at: NaiveDate,
) -> Result<models::OpeningInfo, Box<dyn Error>> {
    Ok(client
        .get(API_TIMES_URL)
        .query(&[("date", at.format(DATE_FORMAT).to_string())])
        .send()
        .await?
        .json()
        .await?)
}

pub async fn get_tables_info(
    client: &Client,
    at: NaiveDate,
    from: String,
    to: String,
) -> Result<Vec<models::TableInfo>, Box<dyn Error>> {
    Ok(client
        .get(API_TABLES_URL)
        .query(&[
            ("date", at.format(DATE_FORMAT).to_string()),
            ("time", format!("{}-{}", from, to)),
        ])
        .send()
        .await?
        .json()
        .await?)
}
