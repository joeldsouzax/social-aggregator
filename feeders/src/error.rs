use rdkafka::error::KafkaError;
use social_engine::error::Error as EngineError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Access Token is empty for `{service}`")]
    EmptyAccessToken { service: String },

    #[error("Failed to initialize `{service}` client")]
    FailedToInitialize { service: String, reason: String },

    #[error(transparent)]
    Producer(#[from] KafkaError),

    #[error(transparent)]
    Engine(#[from] EngineError),
}
