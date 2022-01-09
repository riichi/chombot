use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Player {
    id: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    nickname: String,
    discord_id: String,
}

impl Player {
    fn name(&self) -> String {
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

    fn short_name(&self) -> String {
        if !self.nickname.is_empty() {
            self.nickname.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chombo {
    timestamp: DateTime<Utc>,
    player: String,
    #[serde(default)]
    comment: String,
}

impl Chombo {
    pub fn new(timestamp: DateTime<Utc>, player: &str, comment: &str) -> Self {
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
            id: "".to_string(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "C".to_string(),
            discord_id: "".to_string(),
        };
        assert_eq!(player.name(), "A B (C)");
    }

    #[test]
    fn name_should_return_full_name() {
        let player = Player {
            id: "".to_string(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "".to_string(),
            discord_id: "".to_string(),
        };
        assert_eq!(player.name(), "A B");
    }

    #[test]
    fn name_should_return_nickname() {
        let player = Player {
            id: "".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            nickname: "C".to_string(),
            discord_id: "".to_string(),
        };
        assert_eq!(player.name(), "C");
    }

    #[test]
    fn short_name_should_return_nickname() {
        let player = Player {
            id: "".to_string(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "C".to_string(),
            discord_id: "".to_string(),
        };
        assert_eq!(player.short_name(), "C");
    }

    #[test]
    fn short_name_should_return_full_name() {
        let player = Player {
            id: "".to_string(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "".to_string(),
            discord_id: "".to_string(),
        };
        assert_eq!(player.short_name(), "A B");
    }
}
