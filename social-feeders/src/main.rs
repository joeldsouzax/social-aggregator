use anyhow::Result;
use social_feeders::mastodon;
use std::env;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    prelude::*,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let console = fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter);
    tracing_subscriber::registry().with(console).init();

    let url = env::var("MASTODON_URL")?;
    let url = Url::parse(&url)?;
    let token = env::var("MASTODON_ACCESS_TOKEN")?;
    let mastodon = mastodon::Mastodon::new(url, token)?;
    mastodon.stream().await;
    Ok(())
}
