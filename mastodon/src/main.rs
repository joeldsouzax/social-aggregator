use anyhow::Result;
use std::env;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    prelude::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let console = fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter);
    tracing_subscriber::registry().with(console).init();

    let mastodon_url = env::var("MASTODON_URL")?;
    let mastodon_access_token = env::var("MASTODON_ACCESS_TOKEN")?;

    Ok(())
}

// TODO: a client server which creates a websocket request and fetchs statuses.
// TODO: listens to tokio ctrl+c to stop the connection and gracefully stops the server

// should return a stream of posts
pub trait SocialFeeder {}
