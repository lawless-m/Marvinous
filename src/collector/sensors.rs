//! Sensors data collector (lm-sensors)
//!
//! "I think you ought to know I'm feeling very depressed."

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SensorsError {
    #[error("Failed to execute sensors: {0}")]
    ExecutionError(#[from] std::io::Error),
    #[error("Failed to parse sensors output: {0}")]
    ParseError(String),
    #[error("sensors command not found - is lm-sensors installed?")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub chip: String,
    pub sensor: String,
    pub value: f64,
    pub unit: String,
}

/// Collect sensor readings using `sensors -j`
pub fn collect_sensors() -> Result<Vec<SensorReading>, SensorsError> {
    let output = Command::new("sensors")
        .arg("-j")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file") {
            return Err(SensorsError::NotFound);
        }
        return Err(SensorsError::ParseError(format!(
            "sensors failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&stdout)
        .map_err(|e| SensorsError::ParseError(format!("JSON parse error: {}", e)))?;

    let mut readings = Vec::new();

    for (chip_name, chip_data) in data {
        if let Some(chip_obj) = chip_data.as_object() {
            for (sensor_name, sensor_data) in chip_obj {
                // Skip the Adapter field
                if sensor_name == "Adapter" {
                    continue;
                }

                if let Some(sensor_obj) = sensor_data.as_object() {
                    // Look for temperature readings (temp*_input)
                    for (key, value) in sensor_obj {
                        if key.contains("_input") {
                            if let Some(val) = value.as_f64() {
                                let unit = if key.starts_with("temp") {
                                    "°C"
                                } else if key.starts_with("fan") {
                                    "RPM"
                                } else if key.starts_with("in") {
                                    "V"
                                } else if key.starts_with("power") {
                                    "W"
                                } else {
                                    ""
                                };

                                readings.push(SensorReading {
                                    chip: chip_name.clone(),
                                    sensor: sensor_name.clone(),
                                    value: val,
                                    unit: unit.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(readings)
}

impl std::fmt::Display for SensorReading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}: {:.1}{}", self.chip, self.sensor, self.value, self.unit)
    }
}
