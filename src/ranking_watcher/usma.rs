use std::convert::TryFrom;
use std::error::Error;
use std::result;

use reqwest;
use scraper::{node::Element, node::Node, ElementRef, Html, Selector};
use selectors::attr::CaseSensitivity;

const RANKING_URL: &str = "https://ranking.cvgo.re/";

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
pub struct RankingEntry {
    pub pos: u32,
    pub pos_diff: i32,
    pub id: String,
    pub rank: String,
    pub name: String,
    pub address: String,
    pub points: u32,
    pub points_diff: i32,
}

pub type Ranking = Vec<RankingEntry>;

macro_rules! unpack_children {
    ($element:expr, $n:expr) => {
        <[ElementRef; $n]>::try_from(
            $element
                .children()
                .filter_map(ElementRef::wrap)
                .collect::<Vec<ElementRef>>(),
        )
        .map_err(|v| {
            format!(
                "Could not unpack children into {} elements; got {} instead",
                $n,
                v.len()
            )
        })
    };
}

macro_rules! select_all {
    ($selector:expr, $obj:expr) => {
        $obj.select(
            &Selector::parse($selector).expect(format!("Invalid selector: {}", $selector).as_str()),
        )
    };
}

macro_rules! select_one {
    ($selector:expr, $obj:expr) => {
        select_all!($selector, $obj)
            .next()
            .ok_or(format!("Could not find `{}`", $selector))
    };
}

fn first_nonempty_text<'a>(e: &'a ElementRef) -> Result<&'a str> {
    let ret = e
        .text()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .ok_or("No non-empty text nodes found")?;
    Ok(ret)
}

fn first_element_child<'a>(e: &'a ElementRef) -> Result<&'a Element> {
    let ret = e
        .children()
        .find_map(|chld| match chld.value() {
            Node::Element(e) => Some(e),
            _ => None,
        })
        .ok_or("No element children nodes found")?;
    Ok(ret)
}

fn parse_diff_column(diff_column: &ElementRef) -> Result<i32> {
    match first_element_child(diff_column) {
        Ok(element) => {
            let pos_diff: i32 = first_nonempty_text(diff_column)?.parse()?;
            if element.has_class("has-text-danger", CaseSensitivity::AsciiCaseInsensitive) {
                Ok(-pos_diff)
            } else {
                Ok(pos_diff)
            }
        }
        Err(_) => Ok(0),
    }
}

fn parse_pos_cell(pos_cell: &ElementRef) -> Result<(u32, i32)> {
    let columns = select_one!("div.columns", pos_cell)?;
    let [pos_column, diff_column] = unpack_children!(&columns, 2)?;
    Ok((
        first_nonempty_text(&pos_column)?.parse()?,
        parse_diff_column(&diff_column)?,
    ))
}

fn parse_points_cell(points_cell: &ElementRef) -> Result<(u32, i32)> {
    let columns = select_one!("div.columns", points_cell)?;
    let [diff_column, points_column] = unpack_children!(&columns, 2)?;
    Ok((
        first_nonempty_text(&points_column)?.parse()?,
        parse_diff_column(&diff_column)?,
    ))
}

fn parse_row(row: ElementRef) -> Result<RankingEntry> {
    let [pos_cell, id_cell, rank_cell, player_cell, address_cell, points_cell] =
        unpack_children!(&row, 6)?;
    let (pos, pos_diff) = parse_pos_cell(&pos_cell)?;
    let id = first_nonempty_text(&id_cell).unwrap_or("");
    let rank = first_nonempty_text(&rank_cell).unwrap_or("");
    let name = first_nonempty_text(&player_cell).unwrap_or("");
    let address = first_nonempty_text(&address_cell).unwrap_or("");
    let (points, points_diff) = parse_points_cell(&points_cell)?;
    Ok(RankingEntry {
        pos,
        pos_diff,
        id: String::from(id),
        rank: String::from(rank),
        name: String::from(name),
        address: String::from(address),
        points,
        points_diff,
    })
}

async fn get_ranking_impl() -> Result<Ranking> {
    let body = reqwest::get(RANKING_URL).await?.text().await?;
    let html = Html::parse_document(body.as_str());
    let table = select_one!("table tbody", html)?;
    select_all!("tr", table)
        .into_iter()
        .map(parse_row)
        .collect()
}

pub async fn get_ranking() -> Option<Ranking> {
    match get_ranking_impl().await {
        Ok(ranking) => Some(ranking),
        Err(err) => {
            println!("Error when fetching ranking: {:?}", err);
            None
        }
    }
}
