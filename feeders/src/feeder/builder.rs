use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use schema_registry_converter::async_impl::proto_raw::ProtoRawEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use std::fmt::Debug;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, instrument};

pub struct FeederBuilder<T>
where
    T: Debug,
{
    producer: Option<FutureProducer>,
    recv: Option<Receiver<T>>,
    sender: Option<FeederQueue<T>>,
}

impl<T> FeederBuilder<T>
where
    T: Debug + Clone,
{
    pub fn new() -> Self {
        Self {
            producer: None,
            recv: None,
            sender: None,
        }
    }

    #[instrument(level = "debug", skip(brokers, self) err)]
    pub fn with_brokers<S: AsRef<str>>(self, brokers: S) -> Result<Self, error::Error> {
        debug!("creating a producer targeted at: {}", brokers.as_ref());
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.as_ref())
            .set("message.timeout.ms", "5000")
            .create()?;
        Ok(Self {
            producer: Some(producer),
            recv: None,
            sender: None,
        })
    }

    pub fn with_buffer(self, buffer: usize) -> Self {
        let (tx, rx) = channel::<T>(buffer);
        let producer = self.producer;
        Self {
            producer,
            sender: Some(FeederQueue::create(tx)),
            recv: Some(rx),
        }
    }

    // TODO: returns a feederqueue and Feeder
    pub fn build(self) -> Result<(), error::Error> {
        unimplemented!()
    }
}
