pub mod error;
pub mod mastodon;

use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
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

pub struct Feeder<T>
where
    T: Debug + Clone,
{
    producer: FutureProducer,
    recv: Receiver<T>,
}

impl<T> Feeder<T>
where
    T: Debug + Clone,
{
    pub fn new(producer: FutureProducer, recv: Receiver<T>) -> Self {
        Self { producer, recv }
    }

    #[instrument(level = "debug", skip(self), err)]
    pub async fn run(self, topic: &str) -> Result<(), error::Error> {
        let Feeder { producer, mut recv } = self;
        while let Some(message) = recv.recv().await {
            producer
                .send(
                    FutureRecord::to(topic)
                        .payload(&message)
                        .key(&format!("Key {i}")),
                    Duration::from_secs(0),
                )
                .await
        }
        Ok(())
    }
}

pub struct FeederBuilder<T>
where
    T: Debug + Clone,
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

    pub fn build(self) -> Result<(), error::Error> {
        unimplemented!()
    }
}

pub trait SocialFeeder {
    fn stream<T>(self, queue: FeederQueue<T>) -> impl Future<Output = Result<(), error::Error>>
    where
        T: Debug + Clone;
}
