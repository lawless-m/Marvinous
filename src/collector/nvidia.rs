//! NVIDIA GPU collector
//!
//! "Here I am, brain the size of a planet, and they ask me to watch GPU temperatures."

use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NvidiaError {
    #[error("Failed to execute nvidia-smi: {0}")]
    ExecutionError(#[from] std::io::Error),
    #[error("Failed to parse nvidia-smi output: {0}")]
    ParseError(String),
    #[allow(dead_code)]
    #[error("No NVIDIA GPU detected")]
    NoGpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub name: String,
    pub temperature: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub utilisation: u8,
    pub power_draw: f64,
}

/// Collect GPU status using nvidia-smi
pub fn collect_gpu() -> Result<Option<GpuStatus>, NvidiaError> {
    let output = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // nvidia-smi not found, no GPU
            return Ok(None);
        }
        Err(e) => return Err(NvidiaError::ExecutionError(e)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("NVIDIA-SMI has failed") || stderr.contains("No devices were found") {
            return Ok(None);
        }
        return Err(NvidiaError::ParseError(format!(
            "nvidia-smi failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();

    if line.is_empty() {
        return Ok(None);
    }

    // Parse CSV: name,temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() < 6 {
        return Err(NvidiaError::ParseError(format!(
            "Expected 6 fields, got {}: {}",
            parts.len(),
            line
        )));
    }

    let gpu = GpuStatus {
        name: parts[0].to_string(),
        temperature: parts[1]
            .parse()
            .map_err(|_| NvidiaError::ParseError(format!("Invalid temperature: {}", parts[1])))?,
        memory_used: parts[2]
            .parse()
            .map_err(|_| NvidiaError::ParseError(format!("Invalid memory used: {}", parts[2])))?,
        memory_total: parts[3]
            .parse()
            .map_err(|_| NvidiaError::ParseError(format!("Invalid memory total: {}", parts[3])))?,
        utilisation: parts[4]
            .parse()
            .map_err(|_| NvidiaError::ParseError(format!("Invalid utilisation: {}", parts[4])))?,
        power_draw: parts[5]
            .parse()
            .unwrap_or(0.0), // Power draw might be N/A
    };

    Ok(Some(gpu))
}

impl std::fmt::Display for GpuStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nTemperature: {}°C\nMemory: {} MiB / {} MiB ({}%)\nUtilisation: {}%\nPower: {:.1}W",
            self.name,
            self.temperature,
            self.memory_used,
            self.memory_total,
            (self.memory_used as f64 / self.memory_total as f64 * 100.0) as u8,
            self.utilisation,
            self.power_draw
        )
    }
}
