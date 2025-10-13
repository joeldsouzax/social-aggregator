use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self, Stream};

use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use tracing::instrument;

#[utoipa::path(get,
               path = "/post",
               tags = ["External"],
               operation_id = "post",
               responses(
                   (status = OK, body = String,  description = "A stream of Server-Sent Events (SSE).", content_type = "text/event-stream")
               )
)]
#[instrument(name = "post", target = "api::post")]
pub async fn route() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("some post information"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
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
}
