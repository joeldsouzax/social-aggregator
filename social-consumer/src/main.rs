use anyhow::Result;
use prost::Message;
use proto_definitions::social::v1::{Post, PostBatch};
use redis::{AsyncCommands, RedisResult, aio::MultiplexedConnection};
use social_engine::{engine::SocialEngineBuilder, error::Error};
use std::{env, time::Duration};
use tokio::{sync::mpsc, time::interval};
use tracing::{error, info, instrument};
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
    let kafka_username =
        env::var("KAFKA_USERNAME").expect("Missing required environment variable: KAFKA_USERNAME");
    let kafka_password =
        env::var("KAFKA_PASSWORD").expect("Missing required environment variable: KAFKA_PASSWORD");

    let schema_url = Url::parse(&schema_registry_url)?;
    let consumer = SocialEngineBuilder::decoder(schema_url)
        .with_consumer(&kafka_brokers, &kafka_username, &kafka_password)?
        .build();
    let redis_client = redis::Client::open(redis_url)?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;

    let (tx, mut rx) = mpsc::channel::<Post>(2049);

    let aggregate_task = tokio::spawn({
        let mut redis_publisher = redis_conn.clone();
        let redis_channel = redis_channel.clone();
        async move {
            let mut batch = Vec::with_capacity(50);
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if !batch.is_empty() {
                            info!("timer ticked, publishing {} posts", batch.len());
                            publish_batch(&mut redis_publisher, &redis_channel, &mut batch).await;
                        }
                    }
                    Some(post) = rx.recv() => {
                        batch.push(post);
                        if batch.len() >= 50 {
                            info!("batch full, publishing posts {}", batch.len());
                            publish_batch(&mut redis_publisher, &redis_channel, &mut batch).await;
                        }
                    }
                }
            }
        }
    });

    let topics = [kafka_topic.as_str()];

    let consumer_task = consumer.run(&topics, move |post: Post| {
        let tx = tx.clone();
        async move {
            tx.send(post)
                .await
                .map_err(|e| Error::Generic(e.to_string()))
        }
    });

    info!(
        "Now consuming from '{}' and publishing to Redis channel '{}'",
        kafka_topic, redis_channel
    );
    tokio::try_join!(aggregate_task, async {
        consumer_task.await;
        Ok(())
    })?;
    Ok(())
}

#[instrument]
async fn publish_batch(
    redis_conn: &mut MultiplexedConnection,
    channel: &str,
    batch: &mut Vec<Post>,
) {
    let posts = PostBatch {
        posts: std::mem::take(batch),
    };

    let mut buffer = Vec::new();
    if posts.encode(&mut buffer).is_ok() {
        let result: RedisResult<()> = redis_conn.publish(channel, buffer).await;
        if let Err(e) = result {
            error!("Failed to publish to Redis: {}", e);
        }
    }
}
