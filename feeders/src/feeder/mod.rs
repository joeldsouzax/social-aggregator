pub mod builder;
pub mod queue;

use crate::error::Error;
use queue::FeederQueue;
use rdkafka::producer::FutureProducer;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;
use tracing::instrument;

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
    pub async fn run(self, topic: &str) -> Result<(), Error> {
        let Feeder { producer, mut recv } = self;
        while let Some(message) = recv.recv().await {}
        Ok(())
    }
}

pub trait SocialFeeder {
    type Message: Debug;
    fn stream(self, queue: FeederQueue<Self::Message>) -> impl Future<Output = ()>;
}
