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


#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<TelemetryPoint> {
        vec![
            TelemetryPoint {
                time: 0.0,
                distance: 0.0,
                speed: 100.0,
                throttle: 100.0,
                brake: false,
                gear: 4,
                rpm: 9000,
            },
            TelemetryPoint {
                time: 0.1,
                distance: 20.0,
                speed: 180.0,
                throttle: 100.0,
                brake: false,
                gear: 6,
                rpm: 11000,
            },
            TelemetryPoint {
                time: 0.2,
                distance: 40.0,
                speed: 250.0,
                throttle: 100.0,
                brake: false,
                gear: 7,
                rpm: 12000,
            },
            TelemetryPoint {
                time: 0.3,
                distance: 60.0,
                speed: 220.0,
                throttle: 20.0,
                brake: true,
                gear: 6,
                rpm: 10500,
            },
            TelemetryPoint {
                time: 0.4,
                distance: 80.0,
                speed: 170.0,
                throttle: 0.0,
                brake: true,
                gear: 5,
                rpm: 9000,
            },
            TelemetryPoint {
                time: 0.5,
                distance: 100.0,
                speed: 120.0,
                throttle: 0.0,
                brake: true,
                gear: 4,
                rpm: 7500,
            },
            TelemetryPoint {
                time: 0.6,
                distance: 120.0,
                speed: 135.0,
                throttle: 30.0,
                brake: false,
                gear: 4,
                rpm: 8000,
            },
        ]
    }

    #[test]
    fn max_speed_returns_highest_speed() {
        let points = sample_points();

        let result = max_speed(&points);

        assert_eq!(result, 250.0);
    }

    #[test]
    fn max_speed_returns_zero_for_empty_points() {
        let points = Vec::new();

        let result = max_speed(&points);

        assert_eq!(result, 0.0);
    }

    #[test]
    fn detect_braking_zones_returns_valid_zone() {
        let points = sample_points();

        let zones = detect_braking_zones(&points);

        assert_eq!(zones.len(), 1);

        let zone = &zones[0];

        assert_eq!(zone.start_distance, 60.0);
        assert_eq!(zone.end_distance, 100.0);
        assert_eq!(zone.entry_speed, 220.0);
        assert_eq!(zone.minimum_speed, 120.0);
        assert_eq!(zone.speed_drop, 100.0);
    }

    #[test]
    fn detect_braking_zones_ignores_small_speed_drops() {
        let points = vec![
            TelemetryPoint {
                time: 0.0,
                distance: 0.0,
                speed: 200.0,
                throttle: 80.0,
                brake: false,
                gear: 6,
                rpm: 10000,
            },
            TelemetryPoint {
                time: 0.1,
                distance: 20.0,
                speed: 195.0,
                throttle: 20.0,
                brake: true,
                gear: 6,
                rpm: 9800,
            },
            TelemetryPoint {
                time: 0.2,
                distance: 40.0,
                speed: 188.0,
                throttle: 10.0,
                brake: true,
                gear: 5,
                rpm: 9400,
            },
            TelemetryPoint {
                time: 0.3,
                distance: 60.0,
                speed: 190.0,
                throttle: 40.0,
                brake: false,
                gear: 5,
                rpm: 9500,
            },
        ];

        let zones = detect_braking_zones(&points);

        assert_eq!(zones.len(), 0);
    }

    #[test]
    fn get_driver_file_path_returns_ver_file() {
        let result = get_driver_file_path("VER").unwrap();

        assert_eq!(result, "data/sample_ver.csv");
    }

    #[test]
    fn get_driver_file_path_returns_error_for_unknown_driver() {
        let result = get_driver_file_path("HAM");

        assert!(result.is_err());
    }
}