use aggregator::{ApiDoc, router};
use anyhow::Result;
use axum::{Router, serve::Serve};
use std::{env, net::Ipv4Addr};
use tokio::net::TcpListener;
use tracing::instrument;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    prelude::*,
};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let console = fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter);
    tracing_subscriber::registry().with(console).init();
    let port = env::var("PORT")?;
    let port = port.parse::<u16>()?;
    let _ = app(port).await?;
    Ok(())
}

#[instrument(level = "debug", err)]
async fn app(port: u16) -> Result<Serve<TcpListener, Router, Router>> {
    let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(router())
        .split_for_parts();
    let app = router.merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", api));
    Ok(axum::serve(listener, app))
}
