use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    #[serde(rename = "type")]
    pub kind: ResponseKind,
    pub stream: Stream,
    pub gifts: Vec<Gift>,
    pub debug: DebugInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseKind {
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stream {
    pub user_id: UserId,
    #[serde(rename = "is_private", with = "visibility_as_bool")]
    pub visibility: StreamVisibility,
    pub settings: StreamSettings,
    pub shard_url: Url,
    pub public_tariff: PublicTariff,
    pub private_tariff: PrivateTariff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StreamSettings(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamVisibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicTariff {
    pub id: PublicTariffId,
    pub price: Price,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicTariffId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateTariff {
    pub client_price: Price,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GiftId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gift {
    pub id: GiftId,
    pub price: Price,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugInfo {
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    pub at: DateTime<FixedOffset>,
}

mod visibility_as_bool {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::StreamVisibility;

    pub fn serialize<S>(value: &StreamVisibility, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            StreamVisibility::Public => serializer.serialize_bool(false),
            StreamVisibility::Private => serializer.serialize_bool(true),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StreamVisibility, D::Error>
    where
        D: Deserializer<'de>,
    {
        let is_private = bool::deserialize(deserializer)?;

        if is_private {
            Ok(StreamVisibility::Private)
        } else {
            Ok(StreamVisibility::Public)
        }
    }
}

#[derive(Debug, Error)]
pub enum RequestParseError {
    #[error("failed to deserialize Request from JSON payload in parse_request_json: {0}")]
    DeserializeJson(#[source] serde_json::Error),
}

pub fn parse_request_json(input: &str) -> Result<Request, RequestParseError> {
    serde_json::from_str(input).map_err(RequestParseError::DeserializeJson)
}

#[derive(Debug, Error)]
pub enum RequestFormatError {
    #[error("failed to serialize Request to YAML in request_to_yaml: {0}")]
    SerializeYaml(#[source] serde_yaml::Error),
    #[error("failed to serialize Request to TOML in request_to_toml: {0}")]
    SerializeToml(#[source] toml::ser::Error),
}

pub fn request_to_yaml(request: &Request) -> Result<String, RequestFormatError> {
    serde_yaml::to_string(request).map_err(RequestFormatError::SerializeYaml)
}

pub fn request_to_toml(request: &Request) -> Result<String, RequestFormatError> {
    toml::to_string_pretty(request).map_err(RequestFormatError::SerializeToml)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::{DateTime, FixedOffset};
    use uuid::Uuid;

    use crate::{
        GiftId, Price, PublicTariffId, Request, ResponseKind, StreamVisibility, UserId,
        parse_request_json, request_to_toml, request_to_yaml,
    };

    fn sample_request() -> Request {
        parse_request_json(include_str!("../request.json"))
            .expect("request.json must be valid for tests")
    }

    #[test]
    fn deserializes_request_json_into_typed_model() {
        let request = sample_request();

        assert_eq!(request.kind, ResponseKind::Success);
        assert_eq!(
            request.stream.user_id,
            UserId(
                Uuid::parse_str("8d234120-0bda-49b2-b7e0-fbd3912f6cbf")
                    .expect("valid UUID in fixture")
            )
        );
        assert_eq!(request.stream.visibility, StreamVisibility::Public);
        assert_eq!(request.stream.public_tariff.id, PublicTariffId(1));
        assert_eq!(request.stream.public_tariff.price, Price(100));
        assert_eq!(
            request.stream.public_tariff.duration,
            Duration::from_secs(60 * 60)
        );
        assert_eq!(request.stream.private_tariff.client_price, Price(250));
        assert_eq!(
            request.stream.private_tariff.duration,
            Duration::from_secs(60)
        );
        assert_eq!(request.gifts[0].id, GiftId(1));
        assert_eq!(request.gifts[1].price, Price(3));
        assert_eq!(request.debug.duration, Duration::from_millis(234));
        assert_eq!(
            request.debug.at,
            DateTime::parse_from_rfc3339("2019-06-28T08:35:46+00:00")
                .expect("valid RFC3339 datetime in fixture")
        );
    }

    #[test]
    fn serializes_to_yaml_and_back_without_data_loss() {
        let request = sample_request();
        let yaml = request_to_yaml(&request).expect("yaml serialization should succeed");
        let deserialized: Request =
            serde_yaml::from_str(&yaml).expect("yaml deserialization should succeed");

        assert_eq!(deserialized, request);
    }

    #[test]
    fn serializes_to_toml_and_back_without_data_loss() {
        let request = sample_request();
        let toml = request_to_toml(&request).expect("toml serialization should succeed");
        let deserialized: Request =
            toml::from_str(&toml).expect("toml deserialization should succeed");

        assert_eq!(deserialized, request);
    }

    #[test]
    fn maps_boolean_visibility_to_domain_enum() {
        let request = sample_request();
        let yaml = request_to_yaml(&request).expect("yaml serialization should succeed");

        assert!(yaml.contains("is_private: false"));
    }

    #[test]
    fn parses_datetime_with_fixed_offset() {
        let parsed = DateTime::<FixedOffset>::parse_from_rfc3339("2019-06-28T08:35:46+00:00")
            .expect("test timestamp must parse");

        assert_eq!(
            parsed,
            DateTime::parse_from_rfc3339("2019-06-28T08:35:46+00:00")
                .expect("same RFC3339 timestamp must parse")
        );
    }
}
