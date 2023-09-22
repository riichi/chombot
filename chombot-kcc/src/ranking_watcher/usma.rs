use std::convert::TryFrom;

use anyhow::{anyhow, Context, Result};
use chombot_common::scraping_utils::first_nonempty_text;
use chombot_common::{select_all, select_one, unpack_children};
use reqwest;
use scraper::node::{Element, Node};
use scraper::{CaseSensitivity, ElementRef, Html, Selector};

const RANKING_URL: &str = "https://ranking.cvgo.re/";

#[derive(Debug, Eq, PartialEq)]
pub enum PositionChangeInfo {
    New,
    Diff(i32),
}

impl PositionChangeInfo {
    pub const fn has_changed(&self) -> bool {
        !matches!(self, Self::Diff(0))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RankingEntry {
    pub pos: u32,
    pub pos_diff: PositionChangeInfo,
    pub name: String,
    pub points: u32,
    pub points_diff: PositionChangeInfo,
}

impl RankingEntry {
    pub const fn has_changed(&self) -> bool {
        self.points_diff.has_changed() || self.pos_diff.has_changed()
    }
}

pub struct Ranking(pub Vec<RankingEntry>);

impl Ranking {
    pub fn get_changed(&self) -> Vec<&RankingEntry> {
        self.0.iter().filter(|x| x.has_changed()).collect()
    }
}

impl PartialEq for Ranking {
    fn eq(&self, other: &Self) -> bool {
        self.get_changed() == other.get_changed()
    }
}

impl Eq for Ranking {}

fn first_element_child<'a>(e: &'a ElementRef) -> Option<&'a Element> {
    e.children().find_map(|chld| match chld.value() {
        Node::Element(e) => Some(e),
        _ => None,
    })
}

fn parse_diff_column(diff_column: &ElementRef) -> Result<PositionChangeInfo> {
    match first_element_child(diff_column) {
        Some(element) => {
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
                Err(anyhow!(
                    "Unexpected element without expected classes: {element:?}"
                ))
            }
        }
        None => Ok(PositionChangeInfo::Diff(0)),
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
    let [pos_cell, _id_cell, _rank_cell, player_cell, _address_cell, points_cell] =
        unpack_children!(&row, 6).with_context(|| format!("Failed to unpack cells: {:?}", row))?;
    let (pos, pos_diff) = parse_pos_cell(&pos_cell)
        .with_context(|| format!("Failed to parse position cell: {:?}", row))?;
    let name = first_nonempty_text(&player_cell).unwrap_or("");
    let (points, points_diff) = parse_points_cell(&points_cell)
        .with_context(|| format!("Failed to parse points cell: {:?}", row))?;
    Ok(RankingEntry {
        pos,
        pos_diff,
        name: String::from(name),
        points,
        points_diff,
    })
}

fn parse_document(document: &str) -> Result<Ranking> {
    let html = Html::parse_document(document);
    let table = select_one!("table tbody", html)?;
    let entries: Result<Vec<RankingEntry>> = select_all!("tr", table).map(parse_row).collect();
    Ok(Ranking(entries?))
}

pub async fn get_ranking() -> Result<Ranking> {
    let body = reqwest::get(RANKING_URL).await?.text().await?;
    parse_document(&body)
}

#[cfg(test)]
mod tests {
    use crate::ranking_watcher::usma::{parse_document, PositionChangeInfo, RankingEntry};

    #[test]
    fn builds_ranking_changed_from_real_data() {
        let data = include_str!("test_data/ranking.html");
        let ranking = parse_document(data).unwrap();

        assert_eq!(
            ranking.get_changed(),
            vec![
                &RankingEntry {
                    pos: 1,
                    pos_diff: PositionChangeInfo::Diff(0),
                    name: "player-name-001".to_owned(),
                    points: 1966,
                    points_diff: PositionChangeInfo::Diff(3),
                },
                &RankingEntry {
                    pos: 2,
                    pos_diff: PositionChangeInfo::Diff(3),
                    name: "player-name-002".to_owned(),
                    points: 1893,
                    points_diff: PositionChangeInfo::Diff(163),
                },
                &RankingEntry {
                    pos: 3,
                    pos_diff: PositionChangeInfo::Diff(-1),
                    name: "player-name-003".to_owned(),
                    points: 1830,
                    points_diff: PositionChangeInfo::Diff(-60),
                },
                &RankingEntry {
                    pos: 4,
                    pos_diff: PositionChangeInfo::Diff(3),
                    name: "player-name-004".to_owned(),
                    points: 1718,
                    points_diff: PositionChangeInfo::Diff(73),
                },
                &RankingEntry {
                    pos: 5,
                    pos_diff: PositionChangeInfo::Diff(-2),
                    name: "player-name-005".to_owned(),
                    points: 1711,
                    points_diff: PositionChangeInfo::Diff(-94),
                },
                &RankingEntry {
                    pos: 6,
                    pos_diff: PositionChangeInfo::Diff(0),
                    name: "player-name-006".to_owned(),
                    points: 1706,
                    points_diff: PositionChangeInfo::Diff(56),
                },
                &RankingEntry {
                    pos: 7,
                    pos_diff: PositionChangeInfo::Diff(-3),
                    name: "player-name-007".to_owned(),
                    points: 1685,
                    points_diff: PositionChangeInfo::Diff(-66),
                },
                &RankingEntry {
                    pos: 8,
                    pos_diff: PositionChangeInfo::New,
                    name: "player-name-008".to_owned(),
                    points: 1669,
                    points_diff: PositionChangeInfo::Diff(0),
                },
                &RankingEntry {
                    pos: 8,
                    pos_diff: PositionChangeInfo::New,
                    name: "player-name-009".to_owned(),
                    points: 1669,
                    points_diff: PositionChangeInfo::Diff(0),
                },
                &RankingEntry {
                    pos: 10,
                    pos_diff: PositionChangeInfo::Diff(-1),
                    name: "player-name-010".to_owned(),
                    points: 1651,
                    points_diff: PositionChangeInfo::Diff(43),
                },
            ]
        );
    }
}
