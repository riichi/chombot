use serde::de::Error as SerdeError;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct TableInfo {
    pub label: String,
    pub value: String,
    pub capacity: u8,
    pub available: bool,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct TimeRange {
    pub from: u8,
    pub to: u8,
}

#[derive(Debug, Deserialize)]
pub struct OpeningInfo {
    pub range: TimeRange,
}

#[derive(Debug)]
struct TimeRangeDeserializeError(String);

impl Display for TimeRangeDeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not deserialize TimeRange: {}", self.0)
    }
}

impl Error for TimeRangeDeserializeError {}

impl SerdeError for TimeRangeDeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string())
    }
}

impl<'de> Deserialize<'de> for TimeRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeRangeVisitor;

        impl TimeRangeVisitor {
            fn try_parse_fragment<E: SerdeError>(&self, fragment: &str) -> Result<u8, E> {
                fragment
                    .parse()
                    .map_err(|_| E::invalid_value(Unexpected::Str(fragment), self))
            }
        }

        impl<'de> Visitor<'de> for TimeRangeVisitor {
            type Value = TimeRange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "TimeRange(\"hh-hh\")")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                match v.split_once('-') {
                    None => Err(E::invalid_value(Unexpected::Str(v), &self)),
                    Some((from, to)) => {
                        let from: u8 = self.try_parse_fragment(from)?;
                        let to = self.try_parse_fragment(to)?;
                        Ok(Self::Value { from, to })
                    }
                }
            }
        }

        deserializer.deserialize_str(TimeRangeVisitor {})
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::TimeRange;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Foo {
        str_field: String,
        time_range_field: TimeRange,
    }

    #[test]
    fn test_deserialization() {
        let result = serde_json::from_str::<Foo>(
            r#"{"str_field": "somevalue", "time_range_field": "12-23"}"#,
        )
        .unwrap();
        assert_eq!(
            result,
            Foo {
                str_field: String::from("somevalue"),
                time_range_field: TimeRange { from: 12, to: 23 }
            }
        )
    }
}
