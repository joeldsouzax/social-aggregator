mod routes;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub fn router() -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/post", post(routes::logout))
        .layer(TraceLayer::new_for_http())
        .fallback(routes::not_found)
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Aggregator", description = "Social Aggregator",),
    paths(routes::health::route, routes::logout::route,)
)]
pub struct ApiDoc;
