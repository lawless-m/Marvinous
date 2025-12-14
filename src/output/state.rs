//! State management for trend comparison
//!
//! "The first ten million years were the worst."

use crate::collector::{DriveHealth, GpuStatus, SensorReading};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Failed to read state file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse state file: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousState {
    pub timestamp: DateTime<Utc>,
    pub sensors: Vec<SensorReading>,
    pub gpu: Option<GpuStatus>,
    pub drives: Vec<DriveHealth>,
}

impl PreviousState {
    pub fn new(sensors: Vec<SensorReading>, gpu: Option<GpuStatus>, drives: Vec<DriveHealth>) -> Self {
        Self {
            timestamp: Utc::now(),
            sensors,
            gpu,
            drives,
        }
    }
}

/// Load previous state from file
pub fn load_previous(path: &Path) -> Result<Option<PreviousState>, StateError> {
    if !path.exists() {
        tracing::info!("No previous state file found at {:?}", path);
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let state: PreviousState = serde_json::from_str(&content)?;

    tracing::info!(
        "Loaded previous state from {} ({})",
        path.display(),
        state.timestamp
    );

    Ok(Some(state))
}

/// Save current state for next run
pub fn save_current(path: &Path, state: &PreviousState) -> Result<(), StateError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(state)?;
    fs::write(path, content)?;

    tracing::info!("Saved current state to {}", path.display());

    Ok(())
}
