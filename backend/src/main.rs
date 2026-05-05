use axum::{
    extract::{Path, Query},
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

#[derive(Serialize)]
struct ComparisonResponse {
    driver_a: String,
    driver_b: String,
    points_compared: usize,
    driver_a_max_speed: f64,
    driver_b_max_speed: f64,
    max_speed_delta: f64,
    average_speed_delta: f64,
}

#[derive(Deserialize)]
struct CompareQuery {
    driver_a: String,
    driver_b: String,
}

#[derive(Serialize)]
struct BrakingZone {
    start_distance: f64,
    end_distance: f64,
    entry_speed: f64,
    minimum_speed: f64,
    speed_drop: f64,
}

#[derive(Serialize)]
struct BrakingZonesResponse {
    driver: String,
    zones_detected: usize,
    braking_zones: Vec<BrakingZone>,
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

    let file_path = get_driver_file_path(&driver_code)?; 

    let points = load_telemetry_from_csv(file_path)?;

    Ok(Json(DriverTelemetryResponse {
        driver: driver_code,
        points,
    }))
}

async fn compare_drivers(
    Query(query): Query<CompareQuery>,
) -> Result<Json<ComparisonResponse>, AppError> {
    let driver_a = query.driver_a.to_uppercase();
    let driver_b = query.driver_b.to_uppercase();

    let file_a = get_driver_file_path(&driver_a)?;
    let file_b = get_driver_file_path(&driver_b)?;

    let telemetry_a = load_telemetry_from_csv(file_a)?;
    let telemetry_b = load_telemetry_from_csv(file_b)?;

    let points_compared = telemetry_a.len().min(telemetry_b.len());

    if points_compared == 0 {
        return Err(AppError::Internal(
            "Cannot compare empty telemetry data".to_string(),
        ));
    }

    let driver_a_max_speed = max_speed(&telemetry_a);
    let driver_b_max_speed = max_speed(&telemetry_b);

    let mut total_delta = 0.0;
    let mut max_speed_delta = 0.0;

    for index in 0..points_compared {
        let delta = telemetry_a[index].speed - telemetry_b[index].speed;
        total_delta += delta.abs();

        if delta.abs() > max_speed_delta {
            max_speed_delta = delta.abs();
        }
    }

    let average_speed_delta = total_delta / points_compared as f64;

    Ok(Json(ComparisonResponse {
        driver_a,
        driver_b,
        points_compared,
        driver_a_max_speed,
        driver_b_max_speed,
        max_speed_delta,
        average_speed_delta,
    }))
}

async fn get_braking_zones(
    Path(driver): Path<String>,
) -> Result<Json<BrakingZonesResponse>, AppError> {
    let driver_code = driver.to_uppercase();
    let file_path = get_driver_file_path(&driver_code)?;

    let telemetry = load_telemetry_from_csv(file_path)?;
    let braking_zones = detect_braking_zones(&telemetry);

    Ok(Json(BrakingZonesResponse {
        driver: driver_code,
        zones_detected: braking_zones.len(),
        braking_zones,
    }))
}

fn get_driver_file_path(driver_code: &str) -> Result<&'static str, AppError> {
    match driver_code {
        "VER" => Ok("data/sample_ver.csv"),
        "NOR" => Ok("data/sample_nor.csv"),
        _ => Err(AppError::NotFound(format!(
            "Driver '{}' not found",
            driver_code
        ))),
    }
}

fn detect_braking_zones(points: &[TelemetryPoint]) -> Vec<BrakingZone> {
    let mut zones = Vec::new();
    let mut in_zone = false;

    let mut start_distance = 0.0;
    let mut end_distance = 0.0;
    let mut entry_speed = 0.0;
    let mut minimum_speed = 0.0;

    for point in points {
        if point.brake && !in_zone {
            in_zone = true;
            start_distance = point.distance;
            end_distance = point.distance;
            entry_speed = point.speed;
            minimum_speed = point.speed;
        } else if point.brake && in_zone {
            end_distance = point.distance;

            if point.speed < minimum_speed {
                minimum_speed = point.speed;
            }
        } else if !point.brake && in_zone {
            in_zone = false;

            let speed_drop = entry_speed - minimum_speed;

            if speed_drop > 20.0 {
                zones.push(BrakingZone {
                    start_distance,
                    end_distance,
                    entry_speed,
                    minimum_speed,
                    speed_drop,
                });
            }
        }
    }

    if in_zone {
        let speed_drop = entry_speed - minimum_speed;

        if speed_drop > 20.0 {
            zones.push(BrakingZone {
                start_distance,
                end_distance,
                entry_speed,
                minimum_speed,
                speed_drop,
            });
        }
    }

    zones
}

fn max_speed(points: &[TelemetryPoint]) -> f64 {
    points
        .iter()
        .map(|point| point.speed)
        .fold(0.0, f64::max)
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
        .route("/api/compare", get(compare_drivers))
        .route("/api/braking-zones/{driver}", get(get_braking_zones))
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