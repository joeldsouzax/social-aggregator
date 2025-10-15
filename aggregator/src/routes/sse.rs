use crate::AppState;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use axum_extra::TypedHeader;
use futures_util::stream::{self, Stream};
use headers::{Header, HeaderName, HeaderValue};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use tracing::instrument;

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
    let stream = stream::repeat_with(|| Event::default().data("some post information"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

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

// TODO: connect kafka here and get posts from kafka,
