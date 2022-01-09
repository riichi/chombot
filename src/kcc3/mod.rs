use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use data_types::{Chombo, Player};

pub mod data_types;

const API_PREFIX: &'static str = "/api/";
const PLAYERS_ENDPOINT: &'static str = "players/";
const CHOMBOS_ENDPOINT: &'static str = "chombos/";

pub struct Kcc3Client {
    kcc3_url: String,
    client: Client,
}

impl Kcc3Client {
    pub fn new(kcc3_url: String, auth_token: &str) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&format!("Token {}", auth_token)).expect("Invalid auth token value"));
        let client = reqwest::ClientBuilder::new().default_headers(headers).build()?;

        Ok(Self {
            kcc3_url,
            client,
        })
    }

    pub async fn get_players(&self) -> Result<Vec<Player>, reqwest::Error> {
        self.api_call_get(PLAYERS_ENDPOINT).await
    }

    pub async fn get_chombos(&self) -> Result<Vec<Chombo>, reqwest::Error> {
        self.api_call_get(CHOMBOS_ENDPOINT).await
    }

    pub async fn add_chombo(&self, chombo: &Chombo) -> Result<Chombo, reqwest::Error> {
        self.api_call_post(CHOMBOS_ENDPOINT, chombo).await
    }

    async fn api_call_get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, reqwest::Error> {
        let request_url = format!("{}{}{}", self.kcc3_url, API_PREFIX, endpoint);
        let response = self.client
            .get(&request_url)
            .send()
            .await?;

        response.json().await
    }

    async fn api_call_post<T: DeserializeOwned, P: Serialize>(&self, endpoint: &str, payload: P) -> Result<T, reqwest::Error> {
        let request_url = format!("{}{}{}", self.kcc3_url, API_PREFIX, endpoint);
        let response = self.client
            .post(&request_url)
            .json(&payload)
            .send()
            .await?;

        response.json().await
    }
}
