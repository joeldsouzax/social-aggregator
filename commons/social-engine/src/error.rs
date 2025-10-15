use prost::EncodeError;
use rdkafka::error::KafkaError;
use schema_registry_converter::error::SRCError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Producer(#[from] KafkaError),

    #[error(transparent)]
    Encode(#[from] EncodeError),

    #[error(transparent)]
    ServiceRegistry(#[from] SRCError),
}
