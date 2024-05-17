use std::error::Error;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use chombot_common::scraping_utils::{
    create_chombot_http_client, create_chombot_http_client_insecure,
};
use chombot_common::{ChombotPoiseContext, ChombotPoiseUserData};
use poise::serenity_prelude::{CreateAttachment, CreateMessage};
use poise::CreateReply;

#[derive(Debug)]
pub enum FancyTextFetchError {
    FetchError(anyhow::Error),
    ParseError(anyhow::Error),
}

impl Display for FancyTextFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FetchError(err) => {
                write!(f, "Could not fetch the fancy text response: {err}")
            }
            Self::ParseError(err) => {
                write!(f, "Could not parse the fancy text response: {err}")
            }
        }
    }
}

impl Error for FancyTextFetchError {}

#[derive(Debug, Clone, serde::Deserialize)]
struct FancyTextResponse {
    #[serde(rename = "renderLocation")]
    pub render_location: String,
}

const FANCY_TEXT_URL: &str = "https://cooltext.com/PostChange";

/// Generate a fancy animated text.
#[poise::command(slash_command)]
pub async fn fancy_text<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "The text to render"]
    #[max_length = 100]
    text: String,
) -> Result<()> {
    let client = create_chombot_http_client().map_err(FancyTextFetchError::FetchError)?;

    let handle = ctx.say("Generating...").await?;

    let body: FancyTextResponse = client
        .post(FANCY_TEXT_URL)
        .header("Accept", "*/*")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .form(&[
            ("LogoID", "4"),
            ("Boolean1", "on"),
            ("Integer13", "on"),
            ("Text", &text),
        ])
        .send()
        .await
        .map_err(|err| FancyTextFetchError::FetchError(err.into()))?
        .json()
        .await
        .map_err(|err| FancyTextFetchError::ParseError(err.into()))?;

    let image = create_chombot_http_client_insecure()
        .map_err(FancyTextFetchError::FetchError)?
        .get(body.render_location)
        .send()
        .await
        .map_err(|err| FancyTextFetchError::FetchError(err.into()))?
        .bytes()
        .await
        .map_err(|err| FancyTextFetchError::FetchError(err.into()))?;

    let files: Vec<CreateAttachment> =
        vec![CreateAttachment::bytes(image.slice(..), "fancy_text.gif")];
    ctx.channel_id()
        .send_files(&ctx.http(), files, CreateMessage::new())
        .await?;

    handle
        .edit(
            ctx,
            CreateReply::default().content("Generated with `https://cooltext.com/`"),
        )
        .await?;

    Ok(())
}
