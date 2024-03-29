use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tracing_subscriber::EnvFilter;

use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayerBuilder;
use bikeshare::status::station_status;
use bikeshare::ServerState;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    setup_tracing();
    let (prometheus_layer, metrics_handler) = PrometheusMetricLayerBuilder::new()
        .with_ignore_pattern("/metrics")
        .with_default_metrics()
        .build_pair();

    let app = Router::new()
        .route("/status", get(station_status))
        .route("/healthcheck", get(|| async { axum::http::StatusCode::OK }))
        .with_state(Arc::new(
            ServerState::new(std::env::var("DATABASE_URL").ok()).await,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .include_headers(true),
                ),
        )
        .layer(prometheus_layer);

    let metrics = Router::new()
        .route("/metrics", get(|| async move { metrics_handler.render() }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let metrics_addr = "0.0.0.0:9091".parse::<SocketAddr>().unwrap();
    tracing::info!("Metrics endpoint available at {}/metrics", metrics_addr);
    tokio::spawn(async move {
        axum::Server::bind(&metrics_addr)
            .serve(metrics.into_make_service())
            .await
            .unwrap()
    });

    let app_addr = "0.0.0.0:8080".parse::<SocketAddr>().unwrap();
    tracing::info!("Listening on {}", app_addr);
    axum::Server::bind(&app_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn setup_tracing() {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let fmt_layer = tracing_subscriber::fmt::layer();
    let otel_layer = match std::env::var("HONEYCOMB_API_KEY").ok() {
        Some(key) => {
            let mut metadata: HashMap<String, String> = HashMap::with_capacity(1);
            metadata.insert("x-honeycomb-team".into(), key);

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .http()
                        .with_endpoint("https://api.honeycomb.io/v1/traces")
                        .with_headers(metadata),
                )
                .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                    opentelemetry::sdk::resource::Resource::new(vec![
                        SERVICE_NAME.string("bikeshare"),
                    ]),
                ))
                .install_batch(opentelemetry::runtime::Tokio)
                .unwrap();

            Some(tracing_opentelemetry::layer().with_tracer(tracer))
        }
        None => None,
    };
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(otel_layer)
        .init();
}
