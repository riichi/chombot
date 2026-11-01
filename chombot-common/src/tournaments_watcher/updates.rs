use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use itertools::Itertools;
use log::{error, warn};
use poise::serenity_prelude::{EmojiId, MessageId, ReactionType};
use serde::{Deserialize, Serialize};

use crate::tournaments_watcher::ema::TournamentEntry;

const MAX_REMEMBERED_MESSAGES: usize = 50;
pub const MAX_NUMBERED: usize = 10;

/// Numbers a tournament line in an update: the text the line starts with and
/// the matching reaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberMarker {
    pub text: String,
    pub reaction: ReactionType,
}

fn emote_name(index: usize) -> String {
    format!("{}m", index + 1)
}

fn keycap(index: usize) -> String {
    if index == 9 {
        "\u{1f51f}".to_owned()
    } else {
        format!("{}\u{fe0f}\u{20e3}", index + 1)
    }
}

/// The server's `:1m:` … `:10m:` emotes, given as (name, ID, animated), if it
/// has all of them.
#[must_use]
pub fn emote_numbering<'a>(
    emotes: impl IntoIterator<Item = (&'a str, EmojiId, bool)>,
) -> Option<Vec<NumberMarker>> {
    let emotes: Vec<_> = emotes.into_iter().collect();
    (0..MAX_NUMBERED)
        .map(|index| {
            let name = emote_name(index);
            let &(_, id, animated) = emotes.iter().find(|(candidate, ..)| *candidate == name)?;
            let prefix = if animated { "a" } else { "" };
            Some(NumberMarker {
                text: format!("<{prefix}:{name}:{id}>"),
                reaction: ReactionType::Custom {
                    animated,
                    id,
                    name: Some(name),
                },
            })
        })
        .collect()
}

/// 1️⃣ … 🔟
#[must_use]
pub fn keycap_numbering() -> Vec<NumberMarker> {
    (0..MAX_NUMBERED)
        .map(|index| NumberMarker {
            text: keycap(index),
            reaction: ReactionType::Unicode(keycap(index)),
        })
        .collect()
}

/// Which tournament a `:1m:` … `:10m:` or 1️⃣ … 🔟 reaction stands for.
#[must_use]
pub fn reaction_index(reaction: &ReactionType) -> Option<usize> {
    match reaction {
        ReactionType::Custom {
            name: Some(name), ..
        } => (0..MAX_NUMBERED).find(|&index| emote_name(index) == *name),
        // Keycaps may arrive with or without the variation selector.
        ReactionType::Unicode(emoji) => {
            let strip = |text: &str| text.replace('\u{fe0f}', "");
            (0..MAX_NUMBERED).find(|&index| strip(&keycap(index)) == strip(emoji))
        }
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostedTournament {
    pub name: String,
    /// Only new tournaments are posted with their full entry.
    pub entry: Option<TournamentEntry>,
}

impl PostedTournament {
    /// The tournament's current calendar entry, or the posted one if it's
    /// gone. Tournaments can share a name, so entries are matched by place too.
    #[must_use]
    pub fn current_entry(&self, calendar: &[TournamentEntry]) -> Option<TournamentEntry> {
        let current = self.entry.as_ref().map_or_else(
            || calendar.iter().find(|entry| entry.name == self.name),
            |posted| {
                calendar
                    .iter()
                    .find(|entry| entry.name == posted.name && entry.place == posted.place)
                    .or(Some(posted))
            },
        );
        current.cloned()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostedUpdate {
    pub message_id: MessageId,
    pub tournaments: Vec<PostedTournament>,
}

/// The posted TOURNAMENTS UPDATE messages with numbered reactions, saved to a
/// file so they survive restarts.
#[derive(Debug, Default)]
pub struct TournamentUpdates {
    path: Option<PathBuf>,
    updates: Mutex<Vec<PostedUpdate>>,
}

impl TournamentUpdates {
    #[must_use]
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let updates = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|err| {
                warn!("Ignoring unreadable {}: {err}", path.display());
                Vec::new()
            }),
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    warn!("Could not read {}: {err}", path.display());
                }
                Vec::new()
            }
        };

        Self {
            path: Some(path),
            updates: Mutex::new(updates),
        }
    }

    pub fn record(&self, update: PostedUpdate) {
        let snapshot = {
            let mut updates = self.lock();
            updates.push(update);
            let excess = updates.len().saturating_sub(MAX_REMEMBERED_MESSAGES);
            updates.drain(..excess);
            updates.clone()
        };

        if let Some(path) = &self.path {
            let result = serde_json::to_string_pretty(&snapshot)
                .map_err(anyhow::Error::from)
                .and_then(|contents| Ok(fs::write(path, contents)?));
            if let Err(err) = result {
                error!("Could not save {}: {err}", path.display());
            }
        }
    }

    /// Tournament names, most recently posted first.
    #[must_use]
    pub fn recent_tournaments(&self) -> Vec<String> {
        self.lock()
            .iter()
            .rev()
            .flat_map(|update| &update.tournaments)
            .map(|tournament| tournament.name.clone())
            .unique()
            .collect()
    }

    #[must_use]
    pub fn tournament_for_reaction(
        &self,
        message_id: MessageId,
        reaction: &ReactionType,
    ) -> Option<PostedTournament> {
        let index = reaction_index(reaction)?;
        self.lock()
            .iter()
            .find(|update| update.message_id == message_id)?
            .tournaments
            .get(index)
            .cloned()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<PostedUpdate>> {
        self.updates.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    fn custom(name: &str, id: u64) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: EmojiId::new(id),
            name: Some(name.to_owned()),
        }
    }

    fn named(name: &str) -> PostedTournament {
        PostedTournament {
            name: name.to_owned(),
            entry: None,
        }
    }

    #[test]
    fn numbers_with_server_emotes_if_all_exist() {
        let names: Vec<String> = (1..=10).map(|n| format!("{n}m")).collect();
        let emotes: Vec<_> = names
            .iter()
            .zip(101..)
            .map(|(name, id)| (name.as_str(), EmojiId::new(id), name == "3m"))
            .collect();

        let numbering = emote_numbering(emotes.iter().copied()).unwrap();
        assert_eq!(numbering[0].text, "<:1m:101>");
        assert_eq!(numbering[0].reaction, custom("1m", 101));
        assert_eq!(numbering[2].text, "<a:3m:103>");
        assert_eq!(numbering[9].text, "<:10m:110>");

        assert_eq!(emote_numbering(emotes[..9].iter().copied()), None);
    }

    #[test]
    fn recognizes_numbered_reactions() {
        assert_eq!(reaction_index(&custom("1m", 101)), Some(0));
        assert_eq!(reaction_index(&custom("10m", 110)), Some(9));
        assert_eq!(reaction_index(&custom("11m", 111)), None);
        assert_eq!(reaction_index(&custom("1p", 6)), None);
        let unicode = |emoji: &str| reaction_index(&ReactionType::Unicode(emoji.to_owned()));
        assert_eq!(unicode("1\u{fe0f}\u{20e3}"), Some(0));
        assert_eq!(unicode("1\u{20e3}"), Some(0));
        assert_eq!(unicode("\u{1f51f}"), Some(9));
        assert_eq!(unicode("👍"), None);
    }

    #[test]
    fn remembers_posted_updates_across_restarts() {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        {
            let updates = TournamentUpdates::load(path.to_path_buf());
            updates.record(PostedUpdate {
                message_id: MessageId::new(10),
                tournaments: vec![named("Old Cup"), named("Shared Cup")],
            });
            updates.record(PostedUpdate {
                message_id: MessageId::new(20),
                tournaments: vec![named("New Cup"), named("Shared Cup")],
            });
        }

        let updates = TournamentUpdates::load(path.to_path_buf());
        assert_eq!(
            updates.recent_tournaments(),
            vec!["New Cup", "Shared Cup", "Old Cup"]
        );
        assert_eq!(
            updates.tournament_for_reaction(MessageId::new(10), &custom("2m", 102)),
            Some(named("Shared Cup"))
        );
        assert_eq!(
            updates.tournament_for_reaction(MessageId::new(10), &custom("3m", 103)),
            None
        );
        assert_eq!(
            updates.tournament_for_reaction(MessageId::new(30), &custom("1m", 101)),
            None
        );
    }

    #[test]
    fn picks_the_posted_tournament_among_namesakes() {
        let entry = |place: &str| TournamentEntry {
            name: "Pumpkin Tiles 2026".to_owned(),
            url: String::new(),
            rules: "Riichi".to_owned(),
            date: "31 October 2026".to_owned(),
            place: place.to_owned(),
            approval_status: "OK".to_owned(),
            registration_start: String::new(),
            results_status: String::new(),
        };
        let calendar = [entry("Tbilisi"), entry("Yerevan")];
        let posted = PostedTournament {
            name: "Pumpkin Tiles 2026".to_owned(),
            entry: Some(entry("Yerevan")),
        };

        assert_eq!(posted.current_entry(&calendar), Some(entry("Yerevan")));
        assert_eq!(posted.current_entry(&[]), Some(entry("Yerevan")));
        assert_eq!(
            named("Pumpkin Tiles 2026").current_entry(&calendar),
            Some(entry("Tbilisi"))
        );
    }
}
