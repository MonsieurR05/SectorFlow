use axum::{
    Json,
    extract::{Path, Query},
};

use crate::{
    errors::AppError,
    models::{
        BrakingZonesResponse, CompareQuery, ComparisonResponse, Driver, DriverTelemetryResponse,
        HealthResponse,
    },
    telemetry::{detect_braking_zones, get_driver_file_path, load_telemetry_from_csv, max_speed},
};

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "sectorflow-backend".to_string(),
    })
}

pub async fn get_drivers() -> Json<Vec<Driver>> {
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

pub async fn get_telemetry(
    Path(driver): Path<String>,
) -> Result<Json<DriverTelemetryResponse>, AppError> {
    let driver_code = driver.to_uppercase();
    let file_path = get_driver_file_path(&driver_code)?;
    let points = load_telemetry_from_csv(file_path)?;

    Ok(Json(DriverTelemetryResponse {
        driver: driver_code,
        points,
    }))
}

pub async fn compare_drivers(
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

pub async fn get_braking_zones(
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
