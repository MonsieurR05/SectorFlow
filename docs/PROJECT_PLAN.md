# Project Plan

## Project Name

SectorFlow

## Project Type

Recruiter-facing full-stack software engineering portfolio project.

## Purpose

SectorFlow is designed to demonstrate backend engineering, API design, typed data modelling, data processing, and frontend visualisation through a Formula 1 telemetry analysis system.

## Target Audience

The primary audience is technical recruiters, tutors, and software engineering reviewers. The interface should also be understandable to F1 fans who want to compare driver telemetry.

## MVP Statement

A user can select two drivers from a fixed Formula 1 session and compare their lap telemetry through speed, throttle, brake, and braking-zone analysis.

## Core Features

- Health check API
- CSV telemetry loading
- Driver telemetry endpoint
- Driver comparison endpoint
- Speed delta calculation
- Braking zone detection
- JavaScript dashboard
- Telemetry charts
- Clear technical documentation

## Out of Scope for MVP

- Live F1 data
- Authentication
- Database
- Machine learning
- WebSockets
- Race strategy simulation
- Docker deployment

## Success Criteria

- The backend parses telemetry CSV data into typed Rust structs
- The API exposes structured JSON endpoints
- The analysis engine calculates speed deltas and braking zones
- The frontend visualises telemetry clearly
- The project can be run locally using documented setup steps
- The README explains the architecture and engineering trade-offs