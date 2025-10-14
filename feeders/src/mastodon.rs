use crate::FeederQueue;
use crate::{SocialFeeder, error::Error};
use megalodon::{Megalodon, mastodon::Mastodon as MastodonClient, streaming::Message};
use prost_types::Timestamp;
use proto_definitions::social::v1::{Post, Service};
use tracing::{debug, info, instrument, warn};
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
}

impl SocialFeeder for Mastodon {
    type Message = Post;

    #[instrument(level = "debug")]
    async fn stream(self, queue: FeederQueue<Self::Message>) {
        let streaming = self.client.public_streaming().await;
        streaming
            .listen(Box::new(|message| {
                Box::pin({
                    let queue = queue.clone();
                    async move {
                        match message {
                            Message::Update(status) | Message::StatusUpdate(status) => {
                                debug!("receieved status form mastodon: {}", status.id);
                                let post = Post {
                                    id: status.id,
                                    service: Service::Mastodon as i32,
                                    timestamp: Some(Timestamp {
                                        seconds: status.created_at.timestamp(),
                                        nanos: status.created_at.timestamp_subsec_nanos() as i32,
                                    }),
                                    content: status.content,
                                };
                                let _ = queue.send(post).await;
                            }
                            _ => {}
                        }
                    }
                })
            }))
            .await;
    }
}
