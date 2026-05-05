mod errors;
mod models;
mod routes;
mod telemetry;

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use routes::{compare_drivers, get_braking_zones, get_drivers, get_telemetry, health_check};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/drivers", get(get_drivers))
        .route("/api/telemetry/{driver}", get(get_telemetry))
        .route("/api/compare", get(compare_drivers))
        .route("/api/braking-zones/{driver}", get(get_braking_zones))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    println!("SectorFlow backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind server address");

    axum::serve(listener, app).await.expect("Server failed");
}
