use std::net::SocketAddr;

use axum::{routing::get, Router};
use bikeshare::status::station_status;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "debug".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/status", get(station_status))
        .layer(TraceLayer::new_for_http());
    
    let addr = "0.0.0.0:8080".parse::<SocketAddr>().unwrap();
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
