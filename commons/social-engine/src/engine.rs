use super::queue::FeederQueue;
use crate::error::Error;
use prost::Message;
use proto_definitions::PostId;
use rdkafka::{
    config::ClientConfig,
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
use tracing::{debug, instrument};
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

pub struct MultiSocialProducer<'a, T>
where
    T: Debug + Message + PostId,
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
        T: Debug + Message + PostId,
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
