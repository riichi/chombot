use std::fmt::{Debug, Display, Formatter};

use chrono::{DateTime, Utc};
use poise::ChoiceParameter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
            first_name: String::default(),
            last_name: String::default(),
            nickname,
            discord_id,
        }
    }

    pub fn short_name(&self) -> String {
        if self.nickname.is_empty() {
            format!("{} {}", self.first_name, self.last_name)
        } else {
            self.nickname.clone()
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, ChoiceParameter)]
pub enum ChomboWeight {
    #[default]
    #[name = "1"]
    W1,
    #[name = "1.5"]
    W1_5,
    #[name = "2"]
    W2,
    #[name = "2.5"]
    W2_5,
    #[name = "3"]
    W3,
    #[name = "3.5"]
    W3_5,
    #[name = "4"]
    W4,
    #[name = "4.5"]
    W4_5,
    #[name = "5"]
    W5,
    #[name = "5.5"]
    W5_5,
    #[name = "6"]
    W6,
}

impl ChomboWeight {
    pub const fn as_f64(self) -> f64 {
        match self {
            Self::W1 => 1.0,
            Self::W1_5 => 1.5,
            Self::W2 => 2.0,
            Self::W2_5 => 2.5,
            Self::W3 => 3.0,
            Self::W3_5 => 3.5,
            Self::W4 => 4.0,
            Self::W4_5 => 4.5,
            Self::W5 => 5.0,
            Self::W5_5 => 5.5,
            Self::W6 => 6.0,
        }
    }

    fn from_f64(v: f64) -> Self {
        match v {
            1.5 => Self::W1_5,
            2.0 => Self::W2,
            2.5 => Self::W2_5,
            3.0 => Self::W3,
            3.5 => Self::W3_5,
            4.0 => Self::W4,
            4.5 => Self::W4_5,
            5.0 => Self::W5,
            5.5 => Self::W5_5,
            6.0 => Self::W6,
            _ => Self::W1,
        }
    }
}

impl Display for ChomboWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}

impl Serialize for ChomboWeight {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_f64().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChomboWeight {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = String::deserialize(deserializer)?;
        let v: f64 = v.parse().map_err(serde::de::Error::custom)?;
        Ok(Self::from_f64(v))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Chombo {
    pub timestamp: DateTime<Utc>,
    pub player: PlayerId,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub weight: ChomboWeight,
}

impl Chombo {
    pub fn new(
        timestamp: DateTime<Utc>,
        player: &PlayerId,
        comment: &str,
        weight: ChomboWeight,
    ) -> Self {
        Self {
            timestamp,
            player: player.to_owned(),
            comment: comment.to_owned(),
            weight,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::kcc3::data_types::{Chombo, ChomboWeight, DiscordId, Player, PlayerId};

    #[test]
    fn short_name_should_return_nickname() {
        let player = Player {
            id: PlayerId::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: "C".to_string(),
            discord_id: DiscordId::default(),
        };
        assert_eq!(player.short_name(), "C");
    }

    #[test]
    fn short_name_should_return_full_name() {
        let player = Player {
            id: PlayerId::default(),
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            nickname: String::new(),
            discord_id: DiscordId::default(),
        };
        assert_eq!(player.short_name(), "A B");
    }

    #[test]
    fn as_f64() {
        assert_eq!(ChomboWeight::W1.as_f64(), 1.0);
        assert_eq!(ChomboWeight::W1_5.as_f64(), 1.5);
        assert_eq!(ChomboWeight::W2.as_f64(), 2.0);
        assert_eq!(ChomboWeight::W2_5.as_f64(), 2.5);
        assert_eq!(ChomboWeight::W3.as_f64(), 3.0);
        assert_eq!(ChomboWeight::W3_5.as_f64(), 3.5);
        assert_eq!(ChomboWeight::W4.as_f64(), 4.0);
        assert_eq!(ChomboWeight::W4_5.as_f64(), 4.5);
        assert_eq!(ChomboWeight::W5.as_f64(), 5.0);
        assert_eq!(ChomboWeight::W5_5.as_f64(), 5.5);
        assert_eq!(ChomboWeight::W6.as_f64(), 6.0);
    }

    #[test]
    fn from_f64() {
        assert_eq!(ChomboWeight::from_f64(1.0), ChomboWeight::W1);
        assert_eq!(ChomboWeight::from_f64(1.5), ChomboWeight::W1_5);
        assert_eq!(ChomboWeight::from_f64(2.0), ChomboWeight::W2);
        assert_eq!(ChomboWeight::from_f64(2.5), ChomboWeight::W2_5);
        assert_eq!(ChomboWeight::from_f64(3.0), ChomboWeight::W3);
        assert_eq!(ChomboWeight::from_f64(3.5), ChomboWeight::W3_5);
        assert_eq!(ChomboWeight::from_f64(4.0), ChomboWeight::W4);
        assert_eq!(ChomboWeight::from_f64(4.5), ChomboWeight::W4_5);
        assert_eq!(ChomboWeight::from_f64(5.0), ChomboWeight::W5);
        assert_eq!(ChomboWeight::from_f64(5.5), ChomboWeight::W5_5);
        assert_eq!(ChomboWeight::from_f64(6.0), ChomboWeight::W6);
    }

    #[test]
    fn from_f64_unknown_defaults_to_w1() {
        assert_eq!(ChomboWeight::from_f64(0.0), ChomboWeight::W1);
        assert_eq!(ChomboWeight::from_f64(7.0), ChomboWeight::W1);
    }

    #[test]
    fn deserialize_from_string() {
        assert_eq!(
            serde_json::from_str::<ChomboWeight>("\"2.5\"").unwrap(),
            ChomboWeight::W2_5
        );
        assert_eq!(
            serde_json::from_str::<ChomboWeight>("\"1.0\"").unwrap(),
            ChomboWeight::W1
        );
    }

    #[test]
    fn deserialize_invalid_string() {
        assert!(serde_json::from_str::<ChomboWeight>("\"abc\"").is_err());
    }

    #[test]
    fn deserialize_rejects_number() {
        assert!(serde_json::from_str::<ChomboWeight>("2.5").is_err());
    }

    #[test]
    fn deserialize_chombo_from_api_payload() {
        let json = r#"{
            "id": 229,
            "timestamp": "2026-02-15T22:09:03Z",
            "comment": "furiten ron",
            "weight": "1.0",
            "player": "someplayername"
        }"#;
        let chombo: Chombo = serde_json::from_str(json).unwrap();
        assert_eq!(chombo.player, PlayerId("someplayername".to_string()));
        assert_eq!(chombo.comment, "furiten ron");
        assert_eq!(chombo.weight, ChomboWeight::W1);
        assert_eq!(
            chombo.timestamp,
            DateTime::<Utc>::from_naive_utc_and_offset(
                chrono::NaiveDate::from_ymd_opt(2026, 2, 15)
                    .unwrap()
                    .and_hms_opt(22, 9, 3)
                    .unwrap(),
                Utc,
            )
        );
    }
}
