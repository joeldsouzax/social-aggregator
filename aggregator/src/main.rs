use aggregator::{ApiDoc, router};
use anyhow::Result;
use std::{env, net::Ipv4Addr};
use tokio::net::TcpListener;
use tracing::{debug, info, instrument};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    prelude::*,
};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let console = fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter);
    tracing_subscriber::registry().with(console).init();
    let port = env::var("PORT").unwrap_or("3000".to_string());
    debug!("starting service on: {}", port);
    let port = port.parse::<u16>()?;
    let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(router())
        .split_for_parts();
    let app = router.merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", api));
    info!("swagger ui hosted on: http://localhost:{}/swagger", port);
    info!(
        "openapi spec hosted on: http://localhost:{}/api-docs/openapi.json",
        port
    );
    debug!("creating open api routes");
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    debug!("starting {} on {}", name, listener.local_addr().unwrap());
    info!(
        "--------------------🚀🚀🎆{}:{}@{}🎆🚀🚀--------------------\n",
        "social-aggregator", name, version
    );
    axum::serve(listener, app).await?;
    Ok(())
}
