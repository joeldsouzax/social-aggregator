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
    paths(routes::health::route, routes::logout::route,)
)]
pub struct ApiDoc;
