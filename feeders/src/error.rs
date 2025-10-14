use prost::EncodeError;
use rdkafka::error::KafkaError;
use schema_registry_converter::error::SRCError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Access Token is empty for `{service}`")]
    EmptyAccessToken { service: String },

    #[error("Failed to initialize `{service}` client")]
    FailedToInitialize { service: String, reason: String },

    #[error(transparent)]
    Producer(#[from] KafkaError),

    #[error("Could not send error to kafka: `{0}`")]
    FeederSend(String),

    #[error(transparent)]
    Encode(#[from] EncodeError),

    #[error(transparent)]
    ServiceRegistry(#[from] SRCError),
}
