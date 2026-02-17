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
    pub const fn half_points(self) -> u8 {
        match self {
            Self::W1 => 2,
            Self::W1_5 => 3,
            Self::W2 => 4,
            Self::W2_5 => 5,
            Self::W3 => 6,
            Self::W3_5 => 7,
            Self::W4 => 8,
            Self::W4_5 => 9,
            Self::W5 => 10,
            Self::W5_5 => 11,
            Self::W6 => 12,
        }
    }

    fn as_f64(self) -> f64 {
        f64::from(self.half_points()) / 2.0
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from_f64(v: f64) -> Self {
        match (v * 2.0) as u8 {
            3 => Self::W1_5,
            4 => Self::W2,
            5 => Self::W2_5,
            6 => Self::W3,
            7 => Self::W3_5,
            8 => Self::W4,
            9 => Self::W4_5,
            10 => Self::W5,
            11 => Self::W5_5,
            12 => Self::W6,
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
        let v = f64::deserialize(deserializer)?;
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

    pub const fn half_points(&self) -> u8 {
        self.weight.half_points()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

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

    fn chombo_with_weight(weight: ChomboWeight) -> Chombo {
        Chombo::new(Utc::now(), &PlayerId::default(), "", weight)
    }

    #[test]
    fn half_points() {
        assert_eq!(chombo_with_weight(ChomboWeight::W1).half_points(), 2);
        assert_eq!(chombo_with_weight(ChomboWeight::W1_5).half_points(), 3);
        assert_eq!(chombo_with_weight(ChomboWeight::W2).half_points(), 4);
        assert_eq!(chombo_with_weight(ChomboWeight::W2_5).half_points(), 5);
        assert_eq!(chombo_with_weight(ChomboWeight::W3).half_points(), 6);
        assert_eq!(chombo_with_weight(ChomboWeight::W3_5).half_points(), 7);
        assert_eq!(chombo_with_weight(ChomboWeight::W4).half_points(), 8);
        assert_eq!(chombo_with_weight(ChomboWeight::W4_5).half_points(), 9);
        assert_eq!(chombo_with_weight(ChomboWeight::W5).half_points(), 10);
        assert_eq!(chombo_with_weight(ChomboWeight::W5_5).half_points(), 11);
        assert_eq!(chombo_with_weight(ChomboWeight::W6).half_points(), 12);
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
}
