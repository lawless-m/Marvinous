//! SMART drive health collector
//!
//! "The first ten million years were the worst. And the second ten million, they were the worst too."

use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmartError {
    #[error("Failed to execute smartctl: {0}")]
    ExecutionError(#[from] std::io::Error),
    #[error("Failed to parse smartctl output: {0}")]
    ParseError(String),
    #[allow(dead_code)]
    #[error("smartctl not found - is smartmontools installed?")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveHealth {
    pub device: String,
    pub model: String,
    pub reallocated_sectors: u64,
    pub pending_sectors: u64,
    pub temperature: Option<f64>,
    pub power_on_hours: u64,
}

#[derive(Debug, Deserialize)]
struct SmartCtlOutput {
    model_name: Option<String>,
    ata_smart_attributes: Option<AtaSmartAttributes>,
    nvme_smart_health_information_log: Option<NvmeHealthInfo>,
    temperature: Option<TemperatureInfo>,
}

#[derive(Debug, Deserialize)]
struct AtaSmartAttributes {
    table: Option<Vec<SmartAttribute>>,
}

#[derive(Debug, Deserialize)]
struct SmartAttribute {
    id: u8,
    #[allow(dead_code)]
    name: Option<String>,
    raw: Option<RawValue>,
}

#[derive(Debug, Deserialize)]
struct RawValue {
    value: u64,
}

#[derive(Debug, Deserialize)]
struct NvmeHealthInfo {
    temperature: Option<u64>,
    power_on_hours: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TemperatureInfo {
    current: Option<f64>,
}

/// Auto-detect drives to monitor
pub fn detect_drives() -> Result<Vec<String>, SmartError> {
    let output = Command::new("smartctl")
        .args(["--scan", "--json"])
        .output()?;

    if !output.status.success() {
        // Try without --json for older versions
        let output = Command::new("smartctl")
            .arg("--scan")
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.first().map(|s| s.to_string())
            })
            .filter(|d| d.starts_with("/dev/"))
            .collect();

        return Ok(devices);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON scan output
    #[derive(Deserialize)]
    struct ScanOutput {
        devices: Option<Vec<ScanDevice>>,
    }

    #[derive(Deserialize)]
    struct ScanDevice {
        name: String,
    }

    let scan: ScanOutput = serde_json::from_str(&stdout)
        .map_err(|e| SmartError::ParseError(format!("Failed to parse scan output: {}", e)))?;

    let devices = scan
        .devices
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.name)
        .collect();

    Ok(devices)
}

/// Collect SMART data for specified devices
pub fn collect_smart(devices: &[String]) -> Result<Vec<DriveHealth>, SmartError> {
    let devices_to_check = if devices.is_empty() {
        detect_drives()?
    } else {
        devices.to_vec()
    };

    let mut results = Vec::new();

    for device in devices_to_check {
        match collect_drive_health(&device) {
            Ok(health) => results.push(health),
            Err(e) => {
                tracing::warn!("Failed to get SMART data for {}: {}", device, e);
            }
        }
    }

    Ok(results)
}

fn collect_drive_health(device: &str) -> Result<DriveHealth, SmartError> {
    let output = Command::new("smartctl")
        .args(["-A", "--json", device])
        .output()?;

    // smartctl returns non-zero for various reasons, try to parse anyway
    let stdout = String::from_utf8_lossy(&output.stdout);

    let smart: SmartCtlOutput = serde_json::from_str(&stdout)
        .map_err(|e| SmartError::ParseError(format!("JSON parse error for {}: {}", device, e)))?;

    let model = smart.model_name.unwrap_or_else(|| "Unknown".to_string());

    // Handle NVMe drives
    if let Some(nvme) = smart.nvme_smart_health_information_log {
        return Ok(DriveHealth {
            device: device.to_string(),
            model,
            reallocated_sectors: 0, // NVMe doesn't have this concept
            pending_sectors: 0,
            temperature: nvme.temperature.map(|t| t as f64),
            power_on_hours: nvme.power_on_hours.unwrap_or(0),
        });
    }

    // Handle ATA/SATA drives
    let mut reallocated_sectors = 0u64;
    let mut pending_sectors = 0u64;
    let mut temperature = None;
    let mut power_on_hours = 0u64;

    if let Some(attrs) = smart.ata_smart_attributes {
        if let Some(table) = attrs.table {
            for attr in table {
                let raw_val = attr.raw.map(|r| r.value).unwrap_or(0);
                match attr.id {
                    5 => reallocated_sectors = raw_val,   // Reallocated_Sector_Ct
                    9 => power_on_hours = raw_val,        // Power_On_Hours
                    194 => temperature = Some(raw_val as f64), // Temperature_Celsius
                    197 => pending_sectors = raw_val,     // Current_Pending_Sector
                    _ => {}
                }
            }
        }
    }

    // Try alternative temperature source
    if temperature.is_none() {
        if let Some(temp_info) = smart.temperature {
            temperature = temp_info.current;
        }
    }

    Ok(DriveHealth {
        device: device.to_string(),
        model,
        reallocated_sectors,
        pending_sectors,
        temperature,
        power_on_hours,
    })
}

impl std::fmt::Display for DriveHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}\n  Reallocated Sectors: {}\n  Pending Sectors: {}\n  Temperature: {}°C\n  Power On Hours: {}",
            self.device,
            self.model,
            self.reallocated_sectors,
            self.pending_sectors,
            self.temperature.map(|t| format!("{:.0}", t)).unwrap_or_else(|| "N/A".to_string()),
            self.power_on_hours
        )
    }
}
