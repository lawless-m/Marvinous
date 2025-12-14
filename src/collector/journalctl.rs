//! Journalctl log collector
//!
//! "I've been talking to the ship's computer. It hates me."

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JournalError {
    #[error("Failed to execute journalctl: {0}")]
    ExecutionError(#[from] std::io::Error),
    #[error("Failed to parse journal entry: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub priority: u8,
    pub unit: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
struct JournalEntry {
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    realtime_timestamp: Option<String>,
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
    #[serde(rename = "_SYSTEMD_UNIT")]
    systemd_unit: Option<String>,
    #[serde(rename = "SYSLOG_IDENTIFIER")]
    syslog_identifier: Option<String>,
    #[serde(rename = "MESSAGE")]
    message: Option<serde_json::Value>,
}

impl JournalEntry {
    fn into_log_entry(self) -> Option<LogEntry> {
        let timestamp = self.realtime_timestamp.and_then(|ts| {
            ts.parse::<i64>().ok().and_then(|micros| {
                Utc.timestamp_opt(micros / 1_000_000, ((micros % 1_000_000) * 1000) as u32)
                    .single()
            })
        })?;

        let priority = self
            .priority
            .and_then(|p| p.parse().ok())
            .unwrap_or(6);

        let unit = self.systemd_unit.or(self.syslog_identifier);

        let message = match self.message {
            Some(serde_json::Value::String(s)) => s,
            Some(serde_json::Value::Array(arr)) => {
                // Sometimes MESSAGE is an array of bytes
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|b| b as u8 as char))
                    .collect()
            }
            _ => return None,
        };

        Some(LogEntry {
            timestamp,
            priority,
            unit,
            message,
        })
    }
}

/// Collect system logs from journalctl
pub fn collect_system_logs(since: &str, max_priority: u8, max_entries: usize) -> Result<Vec<LogEntry>, JournalError> {
    let output = Command::new("journalctl")
        .args([
            "--since", since,
            &format!("--priority=0..{}", max_priority),
            "--output=json",
            "--no-pager",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JournalError::ParseError(format!(
            "journalctl failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries: Vec<LogEntry> = stdout
        .lines()
        .filter_map(|line| {
            serde_json::from_str::<JournalEntry>(line)
                .ok()
                .and_then(|e| e.into_log_entry())
        })
        .collect();

    // Truncate if needed
    if entries.len() > max_entries {
        entries.truncate(max_entries);
    }

    Ok(entries)
}

/// Collect kernel logs from journalctl
pub fn collect_kernel_logs(since: &str, max_entries: usize) -> Result<Vec<LogEntry>, JournalError> {
    let output = Command::new("journalctl")
        .args([
            "-k",
            "--since", since,
            "--output=json",
            "--no-pager",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JournalError::ParseError(format!(
            "journalctl -k failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries: Vec<LogEntry> = stdout
        .lines()
        .filter_map(|line| {
            serde_json::from_str::<JournalEntry>(line)
                .ok()
                .and_then(|e| e.into_log_entry())
        })
        .collect();

    // Truncate if needed
    if entries.len() > max_entries {
        entries.truncate(max_entries);
    }

    Ok(entries)
}

impl std::fmt::Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit_str = self.unit.as_deref().unwrap_or("unknown");
        write!(
            f,
            "{} [{}] {}: {}",
            self.timestamp.format("%b %d %H:%M:%S"),
            self.priority,
            unit_str,
            self.message
        )
    }
}
