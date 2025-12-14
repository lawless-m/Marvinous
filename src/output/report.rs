//! Report writing and severity parsing
//!
//! "Life? Don't talk to me about life."

use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("Failed to create report directory: {0}")]
    DirectoryError(std::io::Error),
    #[error("Failed to write report: {0}")]
    WriteError(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Ok,
    Watch,
    Concern,
    Critical,
    Unknown,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Ok => write!(f, "OK"),
            Severity::Watch => write!(f, "WATCH"),
            Severity::Concern => write!(f, "CONCERN"),
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Write report to file
pub fn write_report(
    report_dir: &Path,
    timestamp: DateTime<Utc>,
    content: &str,
) -> Result<PathBuf, ReportError> {
    // Ensure directory exists
    fs::create_dir_all(report_dir).map_err(ReportError::DirectoryError)?;

    // Generate filename: YYYY-MM-DD-HH.md
    let filename = format!("{}.md", timestamp.format("%Y-%m-%d-%H"));
    let path = report_dir.join(filename);

    fs::write(&path, content).map_err(ReportError::WriteError)?;

    tracing::info!("Report written to {}", path.display());

    Ok(path)
}

/// Parse severity from report content
pub fn parse_severity(content: &str) -> Severity {
    // Look for severity in the Summary section
    // Format: "## Summary\n[SEVERITY]: ..."

    let content_upper = content.to_uppercase();

    // Check for severity markers in order of importance
    if content_upper.contains("CRITICAL:") || content_upper.contains("CRITICAL]") {
        return Severity::Critical;
    }
    if content_upper.contains("CONCERN:") || content_upper.contains("CONCERN]") {
        return Severity::Concern;
    }
    if content_upper.contains("WATCH:") || content_upper.contains("WATCH]") {
        return Severity::Watch;
    }
    if content_upper.contains("OK:") || content_upper.contains("OK]") {
        return Severity::Ok;
    }

    Severity::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_severity_ok() {
        let content = r#"# Marvinous Report: 2025-12-14 15:00

## Summary
OK: *Sigh.* Everything's fine. Not that anyone cares.
"#;
        assert_eq!(parse_severity(content), Severity::Ok);
    }

    #[test]
    fn test_parse_severity_critical() {
        let content = r#"# Marvinous Report: 2025-12-14 15:00

## Summary
CRITICAL: Multiple issues require immediate attention.
"#;
        assert_eq!(parse_severity(content), Severity::Critical);
    }

    #[test]
    fn test_parse_severity_watch() {
        let content = r#"# Marvinous Report: 2025-12-14 15:00

## Summary
WATCH: Temperatures are elevated.
"#;
        assert_eq!(parse_severity(content), Severity::Watch);
    }

    #[test]
    fn test_parse_severity_concern() {
        let content = r#"# Marvinous Report: 2025-12-14 15:00

## Summary
CONCERN: Storage health degradation detected.
"#;
        assert_eq!(parse_severity(content), Severity::Concern);
    }
}
