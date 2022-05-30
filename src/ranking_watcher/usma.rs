use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Display;
use std::result;

use reqwest;
use scraper::{node::Element, node::Node, ElementRef, Html, Selector};
use selectors::attr::CaseSensitivity;

const RANKING_URL: &str = "https://ranking.cvgo.re/";

type Result<T> = result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PositionChangeInfo {
    New,
    Diff(i32),
}

const NO_CHANGE: PositionChangeInfo = PositionChangeInfo::Diff(0);

#[derive(Clone, Debug)]
pub struct RankingEntry {
    pub pos: u32,
    pub pos_diff: PositionChangeInfo,
    pub id: String,
    pub rank: String,
    pub name: String,
    pub address: String,
    pub points: u32,
    pub points_diff: PositionChangeInfo,
}

impl PartialEq for RankingEntry {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.pos,
            &self.pos_diff,
            &self.name,
            &self.points,
            &self.points_diff,
        ) == (
            &other.pos,
            &other.pos_diff,
            &other.name,
            &other.points,
            &other.points_diff,
        )
    }
}

impl Eq for RankingEntry {}

pub struct Ranking(pub Vec<RankingEntry>);

impl Ranking {
    pub fn get_changed(&self) -> Self {
        Self(
            self.0
                .iter()
                .filter(|x| x.pos_diff != NO_CHANGE || x.points_diff != NO_CHANGE)
                .cloned()
                .collect(),
        )
    }
}

impl PartialEq for Ranking {
    fn eq(&self, other: &Self) -> bool {
        self.get_changed() == other.get_changed()
    }
}

impl Eq for Ranking {}

#[derive(Debug)]
struct ParseError {
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl Error for ParseError {}

#[derive(Debug)]
pub struct RankingFetchError {
    pub cause: Box<dyn Error + Send + Sync>,
}

impl Display for RankingFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RankingFetchError: {:?}", self.cause.as_ref())
    }
}

impl Error for RankingFetchError {}

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
        $obj.select(&Selector::parse($selector).expect(concat!("Invalid selector: ", $selector)))
    };
}

macro_rules! select_one {
    ($selector:expr, $obj:expr) => {
        select_all!($selector, $obj)
            .next()
            .ok_or(concat!("Could not find any ", $selector))
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

fn parse_diff_column(diff_column: &ElementRef) -> Result<PositionChangeInfo> {
    match first_element_child(diff_column) {
        Ok(element) => {
            if element.has_class("has-text-danger", CaseSensitivity::AsciiCaseInsensitive) {
                Ok(PositionChangeInfo::Diff(
                    -first_nonempty_text(diff_column)?.parse()?,
                ))
            } else if element.has_class("has-text-success", CaseSensitivity::AsciiCaseInsensitive) {
                Ok(PositionChangeInfo::Diff(
                    first_nonempty_text(diff_column)?.parse()?,
                ))
            } else if element.has_class("has-text-info", CaseSensitivity::AsciiCaseInsensitive) {
                Ok(PositionChangeInfo::New)
            } else {
                Err(Box::new(ParseError {
                    message: format!("Unexpected element without expected classes: {:?}", element),
                }))
            }
        }
        Err(_) => Ok(PositionChangeInfo::Diff(0)),
    }
}

fn parse_pos_cell(pos_cell: &ElementRef) -> Result<(u32, PositionChangeInfo)> {
    let columns = select_one!("div.columns", pos_cell)?;
    let [pos_column, diff_column] = unpack_children!(&columns, 2)?;
    Ok((
        first_nonempty_text(&pos_column)?.parse()?,
        parse_diff_column(&diff_column)?,
    ))
}

fn parse_points_cell(points_cell: &ElementRef) -> Result<(u32, PositionChangeInfo)> {
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
    let entries: Result<Vec<RankingEntry>> = select_all!("tr", table)
        .into_iter()
        .map(parse_row)
        .collect();
    Ok(Ranking(entries?))
}

pub async fn get_ranking() -> result::Result<Ranking, RankingFetchError> {
    get_ranking_impl()
        .await
        .map_err(|cause| RankingFetchError { cause })
}
