use anyhow::Result;
use feeders::socials::Mastodon;
use social_engine::{SocialFeeder, engine::SocialEngineBuilder};
use std::env;
use tracing::{info, instrument};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    prelude::*,
};
use url::Url;

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let console = fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter);
    tracing_subscriber::registry().with(console).init();

    info!("🚀 Starting up the social media feeder service...");

    let mastodon_url_str =
        env::var("MASTODON_URL").expect("Missing required environment variable: MASTODON_URL");
    let mastodon_url = Url::parse(&mastodon_url_str)?;
    let mastodon_token = env::var("MASTODON_ACCESS_TOKEN")
        .expect("Missing required environment variable: MASTODON_ACCESS_TOKEN");

    let schema_registry_url_str = env::var("SCHEMA_REGISTRY_URL")
        .expect("Missing required environment variable: SCHEMA_REGISTRY_URL");
    let schema_registry_url = Url::parse(&schema_registry_url_str)?;

    let kafka_brokers =
        env::var("KAFKA_BROKERS").expect("Missing required environment variable: KAFKA_BROKERS");
    let kafka_topic =
        env::var("KAFKA_TOPIC").expect("Missing required environment variable: KAFKA_TOPIC");

    info!(url = %mastodon_url, "Initializing Mastodon feeder client...");
    let mastodon_feeder = Mastodon::new(mastodon_url, mastodon_token)?;

    info!(brokers = %kafka_brokers, "Initializing Kafka producer with schema registry...");
    let (producer, queue) = SocialEngineBuilder::encoder(schema_registry_url)
        .with_producer(&kafka_brokers)?
        .build_multi(100);

    info!(topic = %kafka_topic, "Starting feeder and producer tasks. Streaming live posts...");

    let producer_task = producer.run(&kafka_topic);
    let feeder_task = mastodon_feeder.stream(queue);

    tokio::try_join!(producer_task, async {
        feeder_task.await;
        Ok(())
    })?;
    info!("✅ Service shutting down cleanly.");
    Ok(())
}
