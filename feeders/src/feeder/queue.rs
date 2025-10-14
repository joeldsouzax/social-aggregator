use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use schema_registry_converter::async_impl::proto_raw::ProtoRawEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use std::fmt::Debug;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct FeederQueue<T>
where
    T: Debug,
{
    tx: Sender<T>,
}

impl<T> FeederQueue<T>
where
    T: Debug,
{
    pub fn create(tx: Sender<T>) -> Self {
        Self { tx }
    }

    #[instrument(level = "info", err)]
    pub async fn send(&self, message: T) -> Result<(), error::Error> {
        self.tx
            .send(message)
            .await
            .map_err(|err| error::Error::FeederSend(err.to_string()))
    }
}
