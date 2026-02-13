//! IPMI BMC sensor collector
//!
//! "Life? Don't talk to me about life."

use crate::config::HardwareBaseline;
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

/// Filter IPMI readings based on hardware baseline
/// Removes "no reading" entries for non-installed hardware while preserving them for installed hardware
/// (since "no reading" for installed hardware indicates a failure)
pub fn filter_ipmi_readings(
    readings: Vec<IpmiReading>,
    baseline: &HardwareBaseline,
) -> Vec<IpmiReading> {
    let before_count = readings.len();

    let filtered: Vec<IpmiReading> = readings
        .into_iter()
        .filter(|reading| {
            // If the reading has a valid value, always keep it
            if reading.value != "no reading" && reading.status != "ns" {
                return true;
            }

            // For "no reading" entries, check against baseline
            let sensor_name = &reading.sensor;

            // Check if it's a DIMM sensor
            if sensor_name.starts_with("DIMM_") {
                // Keep if it's in the installed slots (failure detection)
                // Remove if it's not installed (empty slot, expected)
                let is_installed = baseline.memory.installed_slots.contains(sensor_name);
                if is_installed {
                    tracing::warn!("Installed DIMM {} has no reading - possible hardware failure!", sensor_name);
                }
                return is_installed;
            }

            // Check if it's a fan sensor (contains "_FAN" anywhere in the name)
            if sensor_name.contains("_FAN") {
                // Keep if it's in the installed fans (failure detection)
                // Remove if it's not installed (empty header, expected)
                let is_installed = baseline.cooling.installed_fans.contains(sensor_name);
                if is_installed {
                    tracing::warn!("Installed fan {} has no reading - possible hardware failure!", sensor_name);
                }
                return is_installed;
            }

            // For other sensor types, keep them to be conservative
            true
        })
        .collect();

    let filtered_count = before_count - filtered.len();
    tracing::info!(
        "IPMI filtering: {} -> {} sensors ({} filtered)",
        before_count,
        filtered.len(),
        filtered_count
    );

    filtered
}
