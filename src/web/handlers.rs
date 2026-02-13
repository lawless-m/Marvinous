//! HTTP request handlers for web API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::sync::Arc;
use tracing::{error, info, warn};

use super::{
    models::*,
    state::AppState,
};
use crate::output::parse_severity;

/// List all available reports
pub async fn list_reports(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ReportListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let report_dir = &state.config.general.report_dir;

    let entries = match std::fs::read_dir(report_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read report directory: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to read reports: {}", e),
                }),
            ));
        }
    };

    let mut reports = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Parse timestamp from filename (format: YYYY-MM-DD-HH.md)
        let timestamp = parse_filename_timestamp(&filename);

        // Get file size
        let size_bytes = match std::fs::metadata(&path) {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        };

        // Try to extract severity from file content
        let severity = extract_severity(&path).unwrap_or_else(|| "unknown".to_string());

        reports.push(ReportMeta {
            filename,
            timestamp,
            severity,
            size_bytes,
        });
    }

    // Sort by timestamp, newest first
    reports.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total = reports.len();

    Ok(Json(ReportListResponse { reports, total }))
}

/// Get the content of a specific report
pub async fn get_report(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<Json<ReportContent>, (StatusCode, Json<ErrorResponse>)> {
    // Validate filename (prevent path traversal)
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        warn!("Invalid filename requested: {}", filename);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid filename".to_string(),
            }),
        ));
    }

    if !filename.ends_with(".md") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Only .md files allowed".to_string(),
            }),
        ));
    }

    let report_path = state.config.general.report_dir.join(&filename);

    let content = match std::fs::read_to_string(&report_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read report {}: {}", filename, e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Report not found: {}", filename),
                }),
            ));
        }
    };

    let timestamp = parse_filename_timestamp(&filename);
    let severity = extract_severity(&report_path).unwrap_or_else(|| "unknown".to_string());

    Ok(Json(ReportContent {
        filename,
        timestamp,
        content,
        severity,
    }))
}

/// Trigger a manual collection
pub async fn trigger_collect(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CollectResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Try to acquire the collection lock (non-blocking)
    match state.collection_lock.try_lock() {
        Ok(_guard) => {
            info!("Manual collection triggered via web API");

            // Clone config and state for the background task
            let config = state.config.clone();
            let state_clone = Arc::clone(&state);

            // Spawn collection in background
            tokio::spawn(async move {
                info!("Starting background collection");

                // Import is inside the async block to avoid issues
                use crate::run_collection;

                match run_collection(&config).await {
                    Ok(_) => {
                        info!("Background collection completed successfully");
                        // Update last run timestamp
                        let mut last = state_clone.last_run.lock().await;
                        *last = Some(Utc::now());
                    }
                    Err(e) => {
                        error!("Background collection failed: {}", e);
                    }
                }
            });

            Ok(Json(CollectResponse {
                status: "started".to_string(),
                message: "Collection started in background".to_string(),
            }))
        }
        Err(_) => {
            info!("Collection already running, request ignored");
            Ok(Json(CollectResponse {
                status: "already_running".to_string(),
                message: "A collection is already in progress".to_string(),
            }))
        }
    }
}

/// Get current collection status
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Json<StatusResponse> {
    let running = state.collection_lock.try_lock().is_err();
    let last_run = state.last_run.lock().await.clone();

    Json(StatusResponse { running, last_run })
}

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// Helper functions

/// Parse timestamp from filename (format: YYYY-MM-DD-HH.md or YYYY-MM-DD-DAILY.md)
fn parse_filename_timestamp(filename: &str) -> DateTime<Utc> {
    // Remove .md extension
    let name = filename.trim_end_matches(".md");

    // Handle DAILY files: YYYY-MM-DD-DAILY -> use midnight of that date
    if name.ends_with("-DAILY") {
        let date_part = name.trim_end_matches("-DAILY");
        if let Ok(naive) = NaiveDateTime::parse_from_str(&format!("{}-00-00-00", date_part), "%Y-%m-%d-%H-%M-%S") {
            return DateTime::from_naive_utc_and_offset(naive, Utc);
        }
    }

    // Try to parse YYYY-MM-DD-HH format for hourly reports
    if let Ok(naive) = NaiveDateTime::parse_from_str(&format!("{}-00-00", name), "%Y-%m-%d-%H-%M-%S") {
        return DateTime::from_naive_utc_and_offset(naive, Utc);
    }

    // Fallback to epoch
    DateTime::from_timestamp(0, 0).unwrap()
}

/// Extract severity from report content using shared parser
fn extract_severity(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(parse_severity(&content).to_string().to_lowercase())
}
