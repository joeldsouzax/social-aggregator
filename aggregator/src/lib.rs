mod error;
mod json;
mod routes;

use axum::{
    http::{HeaderValue, Method},
    routing::get,
};
use std::env;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

#[derive(Clone, Debug)]
struct AppState {
    redis_client: redis::Client,
}

pub fn router() -> OpenApiRouter {
    let cors_origin = "http://localhost:5173".parse::<HeaderValue>().unwrap();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let app_state = AppState { redis_client };

    OpenApiRouter::new()
        .route("/sse", get(routes::sse))
        .route("/health", get(routes::health))
        .fallback(routes::not_found)
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origin)
                .allow_methods([Method::GET]),
        )
        .layer(TraceLayer::new_for_http())
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Aggregator", description = "Social Aggregator",),
    paths(routes::health::route, routes::sse::route,)
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
