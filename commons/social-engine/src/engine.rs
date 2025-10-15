use super::queue::FeederQueue;
use crate::error::Error;
use prost::Message as ProstMessage;
use proto_definitions::PostId;
use rdkafka::{
    Message,
    config::ClientConfig,
    consumer::{CommitMode, Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
};

use schema_registry_converter::async_impl::proto_raw::{ProtoRawDecoder, ProtoRawEncoder};
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use std::{
    fmt::{self, Debug},
    time::Duration,
};
use tokio::sync::mpsc::{Receiver, channel};
use tracing::{debug, info, instrument, warn};
use url::Url;

pub trait SocialEngine {}

#[derive(Debug)]
pub struct SocialEncoder<'a> {
    encoder: ProtoRawEncoder<'a>,
}

impl<'a> SocialEngine for SocialEncoder<'a> {}

#[derive(Debug)]
pub struct SocialDecoder<'a> {
    decoder: ProtoRawDecoder<'a>,
}

impl<'a> SocialEngine for SocialDecoder<'a> {}

pub struct SocialProducer<'a> {
    producer: FutureProducer,
    encoder: ProtoRawEncoder<'a>,
}

impl<'a> Debug for SocialProducer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocialProducer")
            .field("producer", &"<FutureProducer>")
            .field("encoder", &self.encoder)
            .finish()
    }
}

impl<'a> SocialEngine for SocialProducer<'a> {}

pub struct SocialConsumer<'a> {
    decoder: ProtoRawDecoder<'a>,
    consumer: StreamConsumer,
}

impl<'a> SocialEngine for SocialConsumer<'a> {}

#[derive(Debug)]
pub struct Start;

impl SocialEngine for Start {}

#[derive(Debug)]
pub struct SocialEngineBuilder<E>
where
    E: SocialEngine,
{
    inner: E,
}

impl SocialEngineBuilder<Start> {
    #[instrument(level = "debug")]
    pub fn encoder<'a>(url: Url) -> SocialEngineBuilder<SocialEncoder<'a>> {
        debug!("setting schema registry at: {}", url);
        let sr_settings = SrSettings::new(url.to_string());
        let encoder = ProtoRawEncoder::new(sr_settings);
        SocialEngineBuilder {
            inner: SocialEncoder { encoder },
        }
    }

    #[instrument(level = "debug")]
    pub fn decoder<'a>(url: Url) -> SocialEngineBuilder<SocialDecoder<'a>> {
        debug!("setting schema registry at: {}", url);
        let sr_settings = SrSettings::new(url.to_string());
        let decoder = ProtoRawDecoder::new(sr_settings);
        SocialEngineBuilder {
            inner: SocialDecoder { decoder },
        }
    }
}

impl<'a> SocialEngineBuilder<SocialEncoder<'a>> {
    // TODO: shold take username and password?
    #[instrument(level = "debug", skip(brokers, self) err)]
    pub fn with_producer<S: AsRef<str>>(
        self,
        brokers: S,
    ) -> Result<SocialEngineBuilder<SocialProducer<'a>>, Error> {
        debug!("creating a producer targeted at: {}", brokers.as_ref());
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.as_ref())
            .set("message.timeout.ms", "5000")
            .create()?;

        let encoder = self.inner.encoder;
        Ok(SocialEngineBuilder {
            inner: SocialProducer { encoder, producer },
        })
    }
}

impl<'a> SocialEngineBuilder<SocialDecoder<'a>> {
    #[instrument(level = "debug", skip(brokers, self) err)]
    pub fn with_consumer<S: AsRef<str>>(
        self,
        brokers: S,
    ) -> Result<SocialEngineBuilder<SocialConsumer<'a>>, Error> {
        let consumer = ClientConfig::new()
            .set("group.id", "testcontainer-rs")
            .set("bootstrap.servers", brokers.as_ref())
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create::<StreamConsumer>()?;

        let decoder = self.inner.decoder;
        Ok(SocialEngineBuilder {
            inner: SocialConsumer { decoder, consumer },
        })
    }
}

impl<'a> SocialEngineBuilder<SocialConsumer<'a>> {
    pub fn build(self) -> SocialConsumer<'a> {
        self.inner
    }
}

impl<'a> SocialConsumer<'a> {
    pub async fn run<T, F, Fut>(self, topics: &[&str], f: F)
    where
        T: Debug + ProstMessage + Default,
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<(), Error>>,
    {
        let SocialConsumer { decoder, consumer } = self;
        consumer
            .subscribe(topics)
            .expect("Failed to subscribe to Kafka topics");

        info!(
            "Consumer started. Listening for messages on topics: {:?}",
            topics
        );

        loop {
            match consumer.recv().await {
                Err(e) => warn!("Kafka error: {}", e),
                Ok(m) => {
                    let Some(_) = m.payload() else {
                        warn!("Received message with empty payload, skipping.");
                        continue;
                    };
                    let decoded_message = match decoder.decode(m.payload()).await {
                        Ok(msg) => msg,
                        Err(e) => {
                            warn!("Schema registry decoding error: {}. Skipping message.", e);
                            continue;
                        }
                    };

                    let Some(decoded_message) = decoded_message else {
                        warn!("Received message with empty payload, skipping.");
                        continue;
                    };
                    let final_message = match T::decode(&*decoded_message.bytes) {
                        Ok(msg) => msg,
                        Err(e) => {
                            warn!("Protobuf decoding error: {}. Skipping message.", e);
                            continue;
                        }
                    };

                    info!("Successfully decoded message: {:?}", final_message);
                    if let Err(e) = f(final_message).await {
                        warn!(
                            "Error processing message: {}. Message will not be committed.",
                            e
                        );
                        continue;
                    }
                    if let Err(e) = consumer.commit_message(&m, CommitMode::Async) {
                        warn!("Failed to commit offset: {}", e);
                    }
                }
            };
        }
    }
}

pub struct MultiSocialProducer<'a, T>
where
    T: Debug + ProstMessage + PostId,
{
    producer: FutureProducer,
    encoder: ProtoRawEncoder<'a>,
    recv: Receiver<T>,
}

impl<'a> SocialEngineBuilder<SocialProducer<'a>> {
    pub fn build(self) -> SocialProducer<'a> {
        self.inner
    }

    pub fn build_multi<T>(self, buffer: usize) -> (MultiSocialProducer<'a, T>, FeederQueue<T>)
    where
        T: Debug + ProstMessage + PostId,
    {
        let SocialProducer { producer, encoder } = self.inner;
        let (tx, rx) = channel::<T>(buffer);
        (
            MultiSocialProducer {
                producer,
                encoder,
                recv: rx,
            },
            FeederQueue::create(tx),
        )
    }
}

impl<'a, T> MultiSocialProducer<'a, T>
where
    T: Debug + Message + PostId,
{
    pub async fn run(self, topic: &str) -> Result<(), Error> {
        let MultiSocialProducer {
            producer,
            encoder,
            mut recv,
        } = self;
        while let Some(post) = recv.recv().await {
            let mut proto_bytes = Vec::new();
            post.encode(&mut proto_bytes)?;
            let subject_strategy = SubjectNameStrategy::TopicNameStrategy(topic.to_string(), false);
            let payload = encoder
                .encode(&proto_bytes, "social.v1.Post", subject_strategy)
                .await?;
            producer
                .send(
                    FutureRecord::to(topic).payload(&payload).key(&post.id()),
                    Duration::from_secs(5),
                )
                .await
                .map_err(|(err, _)| err)?;
        }
        Ok(())
    }
}
