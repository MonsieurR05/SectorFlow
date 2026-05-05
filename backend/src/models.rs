use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Serialize)]
pub struct Driver {
    pub code: String,
    pub name: String,
    pub team: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelemetryPoint {
    pub time: f64,
    pub distance: f64,
    pub speed: f64,
    pub throttle: f64,
    pub brake: bool,
    pub gear: i32,
    pub rpm: i32,
}

#[derive(Serialize)]
pub struct DriverTelemetryResponse {
    pub driver: String,
    pub points: Vec<TelemetryPoint>,
}

#[derive(Serialize)]
pub struct ComparisonResponse {
    pub driver_a: String,
    pub driver_b: String,
    pub points_compared: usize,
    pub driver_a_max_speed: f64,
    pub driver_b_max_speed: f64,
    pub max_speed_delta: f64,
    pub average_speed_delta: f64,
}

#[derive(Deserialize)]
pub struct CompareQuery {
    pub driver_a: String,
    pub driver_b: String,
}

#[derive(Serialize)]
pub struct BrakingZone {
    pub start_distance: f64,
    pub end_distance: f64,
    pub entry_speed: f64,
    pub minimum_speed: f64,
    pub speed_drop: f64,
}

#[derive(Serialize)]
pub struct BrakingZonesResponse {
    pub driver: String,
    pub zones_detected: usize,
    pub braking_zones: Vec<BrakingZone>,
}
