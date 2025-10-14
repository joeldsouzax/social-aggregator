pub mod error;
pub mod mastodon;

use rdkafka::{config::ClientConfig, producer::FutureProducer};
use tracing::{debug, instrument};

#[derive(Clone)]
pub struct Feeder {
    pub producer: FutureProducer,
}

impl Feeder {
    #[instrument(level = "debug", skip(brokers) err)]
    pub fn create<S: AsRef<str>>(brokers: S) -> Result<Self, error::Error> {
        debug!("creating a producer targeted at: {}", brokers.as_ref());
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.as_ref())
            .set("message.timeout.ms", "5000")
            .create()?;
        Ok(Feeder { producer })
    }
}

#[cfg(test)]
mod test {
    use crate::Feeder;

    #[test]
    fn test_feeder_creation() {
        let feeder = Feeder::create("localhost:9092");
        assert!(feeder.is_ok());
    }
}
