use std::fs::File;

use crate::{
    errors::AppError,
    models::{BrakingZone, TelemetryPoint},
};

pub fn get_driver_file_path(driver_code: &str) -> Result<&'static str, AppError> {
    match driver_code {
        "VER" => Ok("data/sample_ver.csv"),
        "NOR" => Ok("data/sample_nor.csv"),
        _ => Err(AppError::NotFound(format!(
            "Driver '{}' not found",
            driver_code
        ))),
    }
}

pub fn load_telemetry_from_csv(file_path: &str) -> Result<Vec<TelemetryPoint>, AppError> {
    let file = File::open(file_path)
        .map_err(|_| AppError::Internal(format!("Could not open file: {}", file_path)))?;

    let mut reader = csv::Reader::from_reader(file);
    let mut points = Vec::new();

    for result in reader.deserialize() {
        let point: TelemetryPoint =
            result.map_err(|_| AppError::Internal("Failed to parse telemetry row".to_string()))?;

        points.push(point);
    }

    Ok(points)
}

pub fn max_speed(points: &[TelemetryPoint]) -> f64 {
    points.iter().map(|point| point.speed).fold(0.0, f64::max)
}

pub fn detect_braking_zones(points: &[TelemetryPoint]) -> Vec<BrakingZone> {
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
