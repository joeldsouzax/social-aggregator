use anyhow::Result;
use proto_definitions::social::v1::Post;
use redis::{AsyncCommand, aio::MultiplexedConnection};
use social_engine::engine::SocialEngineBuilder;
use std::{env, time::Duration};
use tokio::{sync::mpsc, time::interval};
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
    info!("🚀 Starting Kafka-to-Redis consumer Service...");

    let kafka_brokers = env::var("KAFKA_BROKERS").expect("KAFKA_BROKERS must be set");
    let schema_registry_url =
        env::var("SCHEMA_REGISTRY_URL").expect("SCHEMA_REGISTRY_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let kafka_topic = env::var("KAFKA_TOPIC").unwrap_or_else(|_| "social.posts".to_string());
    let redis_channel = env::var("REDIS_CHANNEL").unwrap_or_else(|_| "posts.live".to_string());

    let schema_url = Url::parse(&schema_registry_url)?;
    let consumer = SocialEngineBuilder::decoder(schema_url)
        .with_consumer(&kafka_brokers)?
        .build();
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;

    let (tx, mut rx) = mpsc::channel::<Post>(2049);
    let mut redis_publisher = redis_conn.clone();
    let aggregate_task = tokio::spawn(async move {
        let mut batch = Vec::with_capacity(50);
        let mut ticker = interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if !batch.is_empty() {
                        info!("timer ticked, publishing {} posts", batch.len());
                        // TODO: publish
                    }
                }
                Some(post) = rx.recv() => {
                    batch.push(post);
                    if batch.len() >= 50 {
                        info!("batch full, publishing posts {}", batch.len());
                        // TODO: publish
                    }
                }
            }
        }
    });
    Ok(())
}

async fn publish_bastch(
    redis_conn: &mut MultiplexedConnection,
    channel: &str,
    batch: &mut Vec<Post>,
) {
    unimplemented!()
}
