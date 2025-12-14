//! IPMI BMC sensor collector
//!
//! "Life? Don't talk to me about life."

use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpmiError {
    #[error("Failed to execute ipmitool: {0}")]
    ExecutionError(#[from] std::io::Error),
    #[error("ipmitool not found - is ipmitool installed?")]
    NotFound,
    #[error("IPMI modules not loaded")]
    ModulesNotLoaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpmiReading {
    pub sensor: String,
    pub value: String,
    pub status: String,
}

/// Collect IPMI sensor data via ipmitool
pub fn collect_ipmi() -> Result<Vec<IpmiReading>, IpmiError> {
    // Check if ipmitool exists
    if which::which("ipmitool").is_err() {
        return Err(IpmiError::NotFound);
    }

    let output = Command::new("ipmitool")
        .args(["sdr", "list"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Could not open device") || stderr.contains("open failed") {
            return Err(IpmiError::ModulesNotLoaded);
        }
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut readings = Vec::new();

    for line in stdout.lines() {
        // Parse format: "SENSOR_NAME | VALUE | STATUS"
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            readings.push(IpmiReading {
                sensor: parts[0].to_string(),
                value: parts[1].to_string(),
                status: parts[2].to_string(),
            });
        }
    }

    Ok(readings)
}
