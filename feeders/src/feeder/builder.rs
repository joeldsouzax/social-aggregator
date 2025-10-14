use super::queue::FeederQueue;
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use schema_registry_converter::async_impl::proto_raw::ProtoRawEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use std::fmt::Debug;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, instrument};
use url::Url;

pub(crate) trait SocialEngine {}

pub(crate) struct SocialEncoder<'a> {
    encoder: ProtoRawEncoder<'a>,
}

impl<'a> SocialEngine for SocialEncoder<'a> {}

pub(crate) struct SocialProducer<'a> {
    producer: FutureProducer,
    encoder: ProtoRawEncoder<'a>,
}

impl<'a> SocialEngine for SocialProducer<'a> {}

pub struct Start;

impl SocialEngine for Start {}

pub struct SocialEngineBuilder<E>
where
    E: SocialEngine,
{
    inner: E,
}

impl SocialEngineBuilder<Start> {
    #[instrument(level = "debug")]
    pub fn with_encoder_registry<'a>(self, url: Url) -> SocialEngineBuilder<SocialEncoder<'a>> {
        debug!("setting schema registry at: {}", url);
        let sr_settings = SrSettings::new(url.to_string());
        let encoder = ProtoRawEncoder::new(sr_settings);
        SocialEngineBuilder {
            inner: SocialEncoder { encoder },
        }
    }
}

impl<'a> SocialEngineBuilder<SocialEncoder<'a>> {
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
    T: Debug,
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
        T: Debug,
    {
        let SocialProducer { producer, encoder } = self.inner;
        let (tx, rx) = channel::<T>(buffer);
        (
            MultiSocialProducer {
                producer,
                encoder,
                recv: rx,
            },
            FeederQueue::new(tx),
        )
    }
}

impl<'a, T> MultiSocialProducer<'a, T>
where
    T: Debug,
{
    pub async fn run(self, topic: &str) -> Result<(), Error> {
        let MultiSocialProducer {
            producer,
            encoder,
            recv,
        } = self;

        while let Some(message) = recv.recv().await {}
    }
}
