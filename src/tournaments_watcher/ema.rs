use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use anyhow::{anyhow, bail};
use itertools::Itertools;
use scraper::{ElementRef, Html, Selector};

use crate::scraping_utils::{cell_text, first_nonempty_text, select_all, select_one};

const CALENDAR_URL: &str = "http://mahjong-europe.org/ranking/Calendar.html";
const HEADER_CLASS_PREFIX: &str = "TCTT_contenuEntete";
const RCR_RULES_NAME: &str = "Riichi";
const TABLE_COLUMN_NUM: usize = 6;

macro_rules! diff_option_for {
    ($old_object:ident, $new_object:ident, $field_name:ident) => {
        get_diff_option(&$old_object.$field_name, &$new_object.$field_name)
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tournaments(pub Vec<TournamentEntry>);

impl Tournaments {
    #[inline]
    #[must_use]
    const fn get(&self) -> &Vec<TournamentEntry> {
        &self.0
    }

    #[must_use]
    fn into_rcr_only(self) -> Self {
        let filtered = self
            .0
            .into_iter()
            .filter(|entry| entry.rules == RCR_RULES_NAME)
            .collect();
        Self(filtered)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TournamentEntry {
    pub name: String,
    pub url: String,
    pub rules: String,
    pub date: String,
    pub place: String,
    pub approval_status: String,
    pub results_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TournamentChange {
    pub name: String,
    pub url: Option<String>,
    pub rules: Option<String>,
    pub date: Option<String>,
    pub place: Option<String>,
    pub approval_status: Option<String>,
    pub results_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TournamentStatuses(pub Vec<TournamentStatus>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TournamentStatus {
    New(TournamentEntry),
    Changed(TournamentChange),
}

impl TournamentStatus {
    #[must_use]
    fn for_entries(
        old_entry: Option<&TournamentEntry>,
        new_entry: &TournamentEntry,
    ) -> Option<Self> {
        if let Some(old_entry) = old_entry {
            if new_entry == old_entry {
                return None;
            }

            let tournament_change = TournamentChange {
                name: new_entry.name.clone(),
                url: diff_option_for!(old_entry, new_entry, url),
                rules: diff_option_for!(old_entry, new_entry, rules),
                date: diff_option_for!(old_entry, new_entry, date),
                place: diff_option_for!(old_entry, new_entry, place),
                approval_status: diff_option_for!(old_entry, new_entry, approval_status),
                results_status: diff_option_for!(old_entry, new_entry, results_status),
            };

            Some(Self::Changed(tournament_change))
        } else {
            Some(Self::New(new_entry.clone()))
        }
    }
}

#[must_use]
pub fn tournaments_diff(
    entries_old: &Tournaments,
    entries_new: &Tournaments,
) -> TournamentStatuses {
    check_rcr_ruleset(entries_old);
    check_rcr_ruleset(entries_new);

    let mut old_entries_map: HashMap<String, &TournamentEntry> = HashMap::new();
    for entry in unique_entries(entries_old.get()) {
        old_entries_map.insert(entry.name.clone(), entry);
    }

    let mut statuses = Vec::new();
    for new_entry in unique_entries(entries_new.get()) {
        let old_entry = old_entries_map.get(&new_entry.name).copied();
        let status = TournamentStatus::for_entries(old_entry, new_entry);
        if let Some(status) = status {
            statuses.push(status);
        }
    }

    TournamentStatuses(statuses)
}

fn check_rcr_ruleset(entries_old: &Tournaments) {
    assert!(
        entries_old
            .get()
            .iter()
            .all(|entry| entry.rules == RCR_RULES_NAME),
        "Only a Riichi ruleset is supported, please filter your data first"
    );
}

fn unique_entries<'a, T>(entries: T) -> impl Iterator<Item = &'a TournamentEntry>
where
    T: IntoIterator<Item = &'a TournamentEntry>,
{
    entries.into_iter().unique_by(|entry| &entry.name)
}

#[must_use]
fn get_diff_option(value_old: &str, value_new: &str) -> Option<String> {
    if value_old == value_new {
        None
    } else {
        Some(value_new.to_owned())
    }
}

pub fn parse_tournaments(body: &str) -> anyhow::Result<Tournaments> {
    let html = Html::parse_document(body);
    let table = select_one!(".Tableau_CertifiedTournament .TCTT_lignes", html)?;

    let mut last_header = String::new();
    let mut entries = Vec::new();
    for row in select_all!("div", table) {
        let cells: Vec<_> = select_all!("p", row).collect();
        if is_header(&cells) {
            last_header = first_nonempty_text(&cells[0])?.to_owned();
        } else {
            let entry = make_entry(&last_header, &cells)?;
            entries.push(entry);
        }
    }

    Ok(Tournaments(entries))
}

#[must_use]
fn is_header(cells: &[ElementRef]) -> bool {
    let first_cell = cells[0];
    let mut cell_classes = first_cell.value().classes();
    cell_classes.any(|class_name| class_name.starts_with(HEADER_CLASS_PREFIX))
}

fn make_entry(last_header: &str, cells: &[ElementRef]) -> anyhow::Result<TournamentEntry> {
    let url = if let Some(element) = select_all!("a", cells[0]).next() {
        element
            .value()
            .attr("href")
            .ok_or_else(|| anyhow!("<a> element does not contain a link"))?
            .to_owned()
    } else {
        String::new()
    };
    let texts = cells.iter().map(cell_text).collect::<Vec<_>>();
    if texts.len() != TABLE_COLUMN_NUM {
        bail!(
            "Expected {} columns in the EMA tournaments table; got {}",
            TABLE_COLUMN_NUM,
            texts.len()
        );
    }

    let entry = TournamentEntry {
        name: texts[0].clone(),
        url,
        rules: texts[1].clone(),
        date: format!("{} {}", texts[2], last_header),
        place: texts[3].clone(),
        approval_status: texts[4].clone(),
        results_status: texts[5].clone(),
    };
    Ok(entry)
}

pub async fn get_rcr_tournaments() -> Result<Tournaments, TournamentsFetchError> {
    Ok(get_tournaments().await?.into_rcr_only())
}

pub async fn get_tournaments() -> Result<Tournaments, TournamentsFetchError> {
    let body = reqwest::get(CALENDAR_URL)
        .await
        .map_err(|err| TournamentsFetchError::FetchError(err.into()))?
        .text()
        .await
        .map_err(|err| TournamentsFetchError::FetchError(err.into()))?;
    parse_tournaments(&body).map_err(TournamentsFetchError::ParseError)
}

#[derive(Debug)]
pub enum TournamentsFetchError {
    FetchError(anyhow::Error),
    ParseError(anyhow::Error),
}

impl Display for TournamentsFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FetchError(err) => {
                write!(f, "Could not fetch the tournament list: {err}")
            }
            Self::ParseError(err) => {
                write!(f, "Could not parse the tournament list: {err}")
            }
        }
    }
}

impl Error for TournamentsFetchError {}

#[cfg(test)]
mod tests {
    use crate::tournaments_watcher::ema::{
        parse_tournaments, tournaments_diff, TournamentChange, TournamentEntry, TournamentStatus,
    };

    #[test]
    fn builds_diff_from_real_data() {
        let data_1 = include_str!("test_data/calendar_1.html");
        let data_2 = include_str!("test_data/calendar_2.html");
        let entries_1 = parse_tournaments(data_1).unwrap().into_rcr_only();
        let entries_2 = parse_tournaments(data_2).unwrap().into_rcr_only();
        let diffs = tournaments_diff(&entries_1, &entries_2);

        assert_eq!(
            diffs.0,
            vec![
                TournamentStatus::Changed(TournamentChange {
                    name: "Poteto Riichi Taikai 2023".to_owned(),
                    url: None,
                    rules: None,
                    date: Some("1-2 November 2023".to_owned()),
                    place: None,
                    approval_status: None,
                    results_status: Some("Results".to_owned()),
                }),
                TournamentStatus::New(TournamentEntry {
                    name: "Krakow Riichi Open".to_owned(),
                    url: "https://chombo.club".to_owned(),
                    rules: "Riichi".to_owned(),
                    date: "27-31 November 2023".to_owned(),
                    place: "Krakow".to_owned(),
                    approval_status: "OK".to_owned(),
                    results_status: String::new(),
                }),
            ]
        );
    }
}
