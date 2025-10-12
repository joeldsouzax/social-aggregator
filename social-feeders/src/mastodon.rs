use crate::error::Error;
use megalodon::mastodon::Mastodon as MastodonClient;
use tracing::{info, instrument, warn};
use url::Url;

#[derive(Debug, Clone)]
pub struct Mastodon {
    client: MastodonClient,
}

impl Mastodon {
    #[instrument(level = "debug", skip(token), err)]
    pub fn new(url: Url, token: String) -> Result<Self, Error> {
        if token.is_empty() {
            warn!("Attempted to initialize client with an empty access token.");
            return Err(Error::EmptyAccessToken {
                service: "mastodon".to_string(),
            });
        }
        info!("Initializing megalodon client for URL: {}", url);
        let client = MastodonClient::new(url.to_string(), Some(token), None).map_err(|err| {
            Error::FailedToInitialize {
                service: "mastodon".to_string(),
                reason: err.to_string(),
            }
        })?;

        info!("Megalodon client initialized successfully.");
        Ok(Mastodon { client })
    }

    #[instrument(err)]
    pub async fn stream(self) {}
}
