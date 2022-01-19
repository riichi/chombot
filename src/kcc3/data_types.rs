use std::fmt::{Debug, Display, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct PlayerId(pub String);

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct DiscordId(pub String);

impl Display for DiscordId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub id: PlayerId,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub nickname: String,
    pub discord_id: DiscordId,
}

impl Player {
    pub fn new_from_discord(id: PlayerId, nickname: String, discord_id: DiscordId) -> Self {
        Self {
            id,
            first_name: Default::default(),
            last_name: Default::default(),
            nickname,
            discord_id,
        }
    }

    pub fn name(&self) -> String {
        if !self.first_name.is_empty() && !self.last_name.is_empty() {
            let mut s = format!("{} {}", self.first_name, self.last_name);
            if !self.nickname.is_empty() {
                s += &format!(" ({})", self.nickname);
            }

            s
        } else {
            self.nickname.clone()
        }
    }

    pub fn short_name(&self) -> String {
        if !self.nickname.is_empty() {
            self.nickname.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Chombo {
    pub timestamp: DateTime<Utc>,
    pub player: PlayerId,
    #[serde(default)]
    pub comment: String,
}

impl Chombo {
    pub fn new(timestamp: DateTime<Utc>, player: &PlayerId, comment: &str) -> Self {
        Self {
            timestamp,
            player: player.to_owned(),
            comment: comment.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kcc3::data_types::Player;

    #[test]
    fn name_should_return_full_name_with_nickname() {
        let player = Player {
            id: Default::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "C".to_string(),
            discord_id: Default::default(),
        };
        assert_eq!(player.name(), "A B (C)");
    }

    #[test]
    fn name_should_return_full_name() {
        let player = Player {
            id: Default::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "".to_string(),
            discord_id: Default::default(),
        };
        assert_eq!(player.name(), "A B");
    }

    #[test]
    fn name_should_return_nickname() {
        let player = Player {
            id: Default::default(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            nickname: "C".to_string(),
            discord_id: Default::default(),
        };
        assert_eq!(player.name(), "C");
    }

    #[test]
    fn short_name_should_return_nickname() {
        let player = Player {
            id: Default::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "C".to_string(),
            discord_id: Default::default(),
        };
        assert_eq!(player.short_name(), "C");
    }

    #[test]
    fn short_name_should_return_full_name() {
        let player = Player {
            id: Default::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "".to_string(),
            discord_id: Default::default(),
        };
        assert_eq!(player.short_name(), "A B");
    }
}
