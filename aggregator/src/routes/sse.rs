use crate::AppState;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use axum_extra::TypedHeader;
use futures_util::stream::{self, Stream, StreamExt};
use headers::{Header, HeaderName, HeaderValue};
use prost::Message;
use proto_definitions::v1::PostBatch;
use std::{convert::Infallible, env};
use tracing::{error, instrument};

#[utoipa::path(get,
               path = "/sse",
               tags = ["External"],
               operation_id = "sse",
               responses(
                   (status = OK, body = String,  description = "A stream of Server-Sent Events (SSE).", content_type = "text/event-stream")
               )
)]
#[instrument(name = "sse", target = "api::sse")]
pub async fn route(
    State(state): State<AppState>,
    TypedHeader(last_event_id): TypedHeader<LastEventId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let redis_channel = env::var("REDIS_CHANNEL").unwrap_or_else(|_| "posts.live".to_string());
    let mut pubsub = state
        .redis_client
        .get_async_pubsub()
        .await
        .expect("Failed to get redis pubsub connection");
    pubsub
        .subscribe(&redis_channel)
        .await
        .expect("Failed to subscribe to channel");

    let stream = stream::unfold(pubsub, |mut pubsub| async {
        loop {
            let msg = pubsub.on_message().next().await?;
            let payload: Vec<u8> = match msg.get_payload() {
                Ok(payload) => payload,
                Err(e) => {
                    error!("Failed to get payload from Redis message: {}", e);
                    continue;
                }
            };
            let post_batch = match PostBatch::decode(payload.as_slice()) {
                Ok(batch) => batch,
                Err(e) => {
                    error!("Failed to decode Protobuf message: {}", e);
                    continue;
                }
            };
            let json_payload = match serde_json::to_string(&post_batch) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize PostBatch to JSON: {}", e);
                    continue;
                }
            };
            let event = Event::default().data(json_payload);
            return Some((Ok(event), pubsub));
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

static LAST_EVENT_ID: HeaderName = HeaderName::from_static("last-event-id");
#[derive(Debug, Clone)]
pub struct LastEventId(String);

impl Header for LastEventId {
    fn name() -> &'static HeaderName {
        &LAST_EVENT_ID
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let s = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(LastEventId(s.to_owned()))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        if let Ok(value) = HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(value));
        }
    }
}

#[cfg(test)]
mod test {
    // use super::route;
    // use crate::api::test::get_response_body;
    // use axum::http::StatusCode;
    // use axum::response::IntoResponse;
    // use serde_json::json;

    // #[tokio::test]
    // async fn health_gives_ok() {
    //     let response = route().await;
    //     let response = response.into_response();
    //     assert_eq!(response.status(), StatusCode::OK);
    //     let body = get_response_body(response).await;
    //     assert_eq!(
    //         body,
    //         json!({"data": {"message": "ok."}, "status": "success"})
    //     );
    // }
    //
    //
}
