//! API request and response models

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Metadata for a single report file
#[derive(Debug, Serialize)]
pub struct ReportMeta {
    pub filename: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String,
    pub size_bytes: u64,
}

/// Response for listing all reports
#[derive(Debug, Serialize)]
pub struct ReportListResponse {
    pub reports: Vec<ReportMeta>,
    pub total: usize,
}

/// Full report content with metadata
#[derive(Debug, Serialize)]
pub struct ReportContent {
    pub filename: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub severity: String,
}

/// Response from triggering a collection
#[derive(Debug, Serialize)]
pub struct CollectResponse {
    pub status: String,  // "started" | "already_running" | "error"
    pub message: String,
}

/// Current collection status
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub last_run: Option<DateTime<Utc>>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
