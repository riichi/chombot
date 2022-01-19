use std::fmt::{Display, Formatter};

use reqwest::{Client, Error};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Serialize;

use data_types::{Chombo, Player};

pub mod data_types;

const API_PREFIX: &'static str = "/api/";
const PLAYERS_ENDPOINT: &'static str = "players/";
const CHOMBOS_ENDPOINT: &'static str = "chombos/";

#[derive(Debug)]
pub struct Kcc3ClientError {
    inner_error: reqwest::Error,
}

impl Kcc3ClientError {
    fn new(inner_error: reqwest::Error) -> Self {
        return Self {
            inner_error,
        };
    }
}

type Kcc3ClientResult<T> = Result<T, Kcc3ClientError>;

impl Display for Kcc3ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not reach KCC3: {}", self.inner_error)
    }
}

impl From<reqwest::Error> for Kcc3ClientError {
    fn from(e: Error) -> Self {
        Self::new(e)
    }
}

pub struct Kcc3Client {
    kcc3_url: String,
    client: Client,
}

impl Kcc3Client {
    pub fn new(kcc3_url: String, auth_token: &str) -> Kcc3ClientResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&format!("Token {}", auth_token)).expect("Invalid auth token value"));
        let client = reqwest::ClientBuilder::new().default_headers(headers).build()?;

        Ok(Self {
            kcc3_url,
            client,
        })
    }

    pub async fn get_players(&self) -> Kcc3ClientResult<Vec<Player>> {
        self.api_call_get(PLAYERS_ENDPOINT).await
    }

    pub async fn add_player(&self, player: &Player) -> Kcc3ClientResult<Player> {
        self.api_call_post(PLAYERS_ENDPOINT, player).await
    }

    pub async fn get_chombos(&self) -> Kcc3ClientResult<Vec<Chombo>> {
        self.api_call_get(CHOMBOS_ENDPOINT).await
    }

    pub async fn add_chombo(&self, chombo: &Chombo) -> Kcc3ClientResult<Chombo> {
        self.api_call_post(CHOMBOS_ENDPOINT, chombo).await
    }

    async fn api_call_get<T: DeserializeOwned>(&self, endpoint: &str) -> Kcc3ClientResult<T> {
        let request_url = format!("{}{}{}", self.kcc3_url, API_PREFIX, endpoint);
        let response = self.client
            .get(&request_url)
            .send()
            .await?;

        response
            .error_for_status()?
            .json()
            .await
            .map_err(|x| x.into())
    }

    async fn api_call_post<T: DeserializeOwned, P: Serialize>(&self, endpoint: &str, payload: P) -> Kcc3ClientResult<T> {
        let request_url = format!("{}{}{}", self.kcc3_url, API_PREFIX, endpoint);
        let response = self.client
            .post(&request_url)
            .json(&payload)
            .send()
            .await?;

        response
            .error_for_status()?
            .json()
            .await
            .map_err(|x| x.into())
    }
}
