use crate::{error::Error, json::ValidJson};
use axum::http::StatusCode;
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    message: String,
}

#[utoipa::path(get,
               path = "/post",
               tags = ["external"],
               operation_id = "post",
               responses(
                   (status = OK, body = HealthResponse, description = "Streaming posts", content_type = "application/json")
               )
)]
#[instrument(name = "health", target = "api::health")]
pub async fn route() -> Result<(StatusCode, ValidJson<HealthResponse>), Error> {
    Ok((
        StatusCode::OK,
        ValidJson(HealthResponse {
            message: "ok.".to_owned(),
        }),
    ))
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
