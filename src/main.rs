use axum::{routing::get, Router};
use bikeshare::status::station_status;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/status", get(station_status));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
