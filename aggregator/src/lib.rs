mod error;
mod json;
mod routes;

use axum::routing::get;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub fn router() -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/post", get(routes::post))
        .route("/health", get(routes::health))
        .layer(TraceLayer::new_for_http())
        .fallback(routes::not_found)
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Aggregator", description = "Social Aggregator",),
    paths(routes::health::route, routes::post::route,)
)]
pub struct ApiDoc;

#[cfg(test)]
mod test {
    use super::router;
    use axum::{
        Router,
        body::Body,
        http::{Request, Response, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::ServiceExt;

    pub fn get_router() -> Router {
        router().split_for_parts().0
    }

    pub async fn get_response_body(response: Response<Body>) -> Value {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_fallback() {
        let router = get_router();
        let non_existing_route = Request::builder()
            .uri("/fallback")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(non_existing_route).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, String::from_utf8(body.to_vec()).unwrap());
    }
}
