use anyhow::{anyhow, Result};
use itertools::Itertools;
use reqwest::ClientBuilder;
use scraper::ElementRef;

const USER_AGENT: &str = concat!("chombot/", env!("CARGO_PKG_VERSION"));

#[macro_export]
macro_rules! unpack_children {
    ($element:expr, $n:expr) => {
        <[ElementRef; $n]>::try_from(
            $element
                .children()
                .filter_map(ElementRef::wrap)
                .collect::<Vec<ElementRef>>(),
        )
        .map_err(|v| {
            anyhow!(
                "Could not unpack children into {} elements; got {} instead",
                $n,
                v.len()
            )
        })
    };
}

#[macro_export]
macro_rules! select_all {
    ($selector:expr, $obj:expr) => {
        $obj.select(&Selector::parse($selector).expect(concat!("Invalid selector: ", $selector)))
    };
}

#[macro_export]
macro_rules! select_one {
    ($selector:expr, $obj:expr) => {
        select_all!($selector, $obj)
            .next()
            .ok_or(anyhow!(concat!("Could not find any ", $selector)))
    };
}

pub fn first_nonempty_text<'a>(e: &'a ElementRef) -> anyhow::Result<&'a str> {
    let ret = e
        .text()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("No non-empty text nodes found"))?;
    Ok(ret)
}

#[must_use]
pub fn cell_text(e: &ElementRef) -> String {
    e.text().map(str::trim).join(" ").trim().to_owned()
}

pub fn create_chombot_http_client() -> Result<reqwest::Client> {
    Ok(create_chombot_http_client_base().build()?)
}

pub fn create_chombot_http_client_insecure() -> Result<reqwest::Client> {
    Ok(create_chombot_http_client_base()
        .danger_accept_invalid_certs(true)
        .build()?)
}

fn create_chombot_http_client_base() -> ClientBuilder {
    reqwest::Client::builder().user_agent(USER_AGENT)
}
