#![allow(clippy::default_constructed_unit_structs)] // warning since 1.71

use autometrics::{autometrics, prometheus_exporter};
use axum::{response::IntoResponse, routing::get, BoxError, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use serde_json::json;
use std::env;
use std::net::SocketAddr;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    prometheus_exporter::init();

    let app = app();
    let addr = &"0.0.0.0:8080".parse::<SocketAddr>()?;

    if env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_ok() {
        init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
        tracing::info!("listening on {}", addr);
    }

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    let router = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route(
            "/metrics",
            get(|| async { prometheus_exporter::encode_http_response() }),
        );

    if env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_ok() {
        let tracing_router = Router::new()
            .layer(OtelInResponseLayer::default())
            .layer(OtelAxumLayer::default());
        Router::new().merge(router).merge(tracing_router)
    } else {
        router
    }
}

#[autometrics(track_concurrency)]
async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

#[autometrics(track_concurrency)]
#[tracing::instrument]
async fn index() -> impl IntoResponse {
    if env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_ok() {
        let trace_id = find_current_trace_id();
        dbg!(&trace_id);
        axum::Json(json!({ "my_trace_id": trace_id }))
    } else {
        axum::Json(json!({ "my_trace_id": "not here"}))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    if env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_ok() {
        tracing::warn!("signal received, starting graceful shutdown");
        opentelemetry::global::shutdown_tracer_provider();
    }
}
