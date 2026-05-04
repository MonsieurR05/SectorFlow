# SectorFlow

SectorFlow is a full-stack Formula 1 telemetry analysis platform built with a Rust backend and a JavaScript dashboard.

The project processes lap telemetry data, compares driver performance, detects braking zones, and exposes structured API endpoints for visual analysis.

## Project Goals

- Build a Rust-based telemetry processing engine
- Expose clean REST API endpoints using Axum
- Parse and analyse F1 lap telemetry from CSV data
- Compare driver performance using speed, throttle, brake, and distance data
- Visualise telemetry through a JavaScript dashboard
- Demonstrate full-stack software engineering, API design, and data processing

## Planned Tech Stack

### Backend

- Rust
- Axum
- Tokio
- Serde
- CSV crate
- Tower HTTP

### Frontend

- Next.js
- JavaScript
- Tailwind CSS
- Recharts

### Data

- Sample CSV telemetry data for MVP
- FastF1 export script planned for real F1 telemetry data

## MVP Scope

The initial MVP uses sample telemetry fixtures to validate the API design and Rust parsing pipeline. Real Formula 1 telemetry data will be exported using FastF1 and processed through the same CSV-based backend pipeline.

Core MVP features:

- Load telemetry CSV files
- Return telemetry data through a Rust API
- Compare two drivers
- Calculate speed deltas
- Detect braking zones
- Display telemetry charts in a dashboard

## Project Status

Planning and initial setup.
