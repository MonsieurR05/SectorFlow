use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, net::SocketAddr};
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TelemetryPoint {
    time: f64,
    distance: f64,
    speed: f64,
    throttle: f64,
    brake: bool,
    gear: i32,
    rpm: i32,
}

#[derive(Serialize)]
struct DriverTelemetryResponse {
    driver: String,
    points: Vec<TelemetryPoint>,
}

#[derive(Serialize)]
struct Driver {
    code: String,
    name: String,
    team: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "sectorflow-backend".to_string(),
    })
}

async fn get_drivers() -> Json<Vec<Driver>> {
    Json(vec![
        Driver {
            code: "VER".to_string(),
            name: "Max Verstappen".to_string(),
            team: "Red Bull Racing".to_string(),
        },
        Driver {
            code: "NOR".to_string(),
            name: "Lando Norris".to_string(),
            team: "McLaren".to_string(),
        },
    ])
}

async fn get_telemetry(Path(driver): Path<String>) -> Result<Json<DriverTelemetryResponse>, AppError> {
    let driver_code = driver.to_uppercase();

    let file_path = match driver_code.as_str() {
        "VER" => "data/sample_ver.csv",
        "NOR" => "data/sample_nor.csv",
        _ => return Err(AppError::NotFound(format!("Driver '{}' not found", driver_code))),
    };

    let points = load_telemetry_from_csv(file_path)?;

    Ok(Json(DriverTelemetryResponse {
        driver: driver_code,
        points,
    }))
}

fn load_telemetry_from_csv(file_path: &str) -> Result<Vec<TelemetryPoint>, AppError> {
    let file = File::open(file_path)
        .map_err(|_| AppError::Internal(format!("Could not open file: {}", file_path)))?;

    let mut reader = csv::Reader::from_reader(file);
    let mut points = Vec::new();

    for result in reader.deserialize() {
        let point: TelemetryPoint = result
            .map_err(|_| AppError::Internal("Failed to parse telemetry row".to_string()))?;

        points.push(point);
    }

    Ok(points)
}

enum AppError {
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

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
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    println!("SectorFlow backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind server address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}