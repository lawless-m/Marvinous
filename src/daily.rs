//! Daily summary and archiving module
//!
//! "Oh wonderful, now I have to summarize summaries. How delightfully recursive."

use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::config::Config;
use crate::llm::OllamaClient;

#[derive(Error, Debug)]
pub enum DailyError {
    #[error("No reports found for date: {0}")]
    NoReports(String),
    #[error("Failed to read reports: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to create archive: {0}")]
    ArchiveError(String),
    #[error("LLM error: {0}")]
    LlmError(String),
}

/// Generate daily summary from previous day's hourly reports
pub async fn generate_daily_summary(config: &Config) -> Result<(), DailyError> {
    // Get yesterday's date
    let yesterday = Utc::now() - chrono::Duration::days(1);
    let date = yesterday.format("%Y-%m-%d").to_string();

    info!("Generating daily summary for {}", date);

    // Find all hourly reports from yesterday
    let reports = find_hourly_reports(&config.general.report_dir, &date)?;

    if reports.is_empty() {
        return Err(DailyError::NoReports(date));
    }

    info!("Found {} hourly reports for {}", reports.len(), date);

    // Read all report contents
    let mut report_contents = Vec::new();
    for report_path in &reports {
        let content = fs::read_to_string(report_path)?;
        report_contents.push(content);
    }

    // Build daily summary prompt
    let prompt = build_daily_prompt(&report_contents, &date);

    // Generate summary using LLM
    let client = OllamaClient::new(
        &config.ollama.endpoint,
        &config.ollama.model,
        config.ollama.timeout_secs,
    );

    info!("Sending daily summary prompt to LLM ({} chars)", prompt.len());

    let summary = client
        .generate(&prompt)
        .await
        .map_err(|e| DailyError::LlmError(e.to_string()))?;

    // Write daily summary
    let summary_path = write_daily_summary(&config.general.report_dir, &date, &summary)?;
    info!("Daily summary written to: {}", summary_path.display());

    // Archive hourly reports
    let archive_path = archive_hourly_reports(&config.general.report_dir, &date, &reports)?;
    info!("Hourly reports archived to: {}", archive_path.display());

    // Delete hourly reports after successful archiving
    for report_path in &reports {
        if let Err(e) = fs::remove_file(report_path) {
            warn!("Failed to delete {}: {}", report_path.display(), e);
        }
    }

    info!("Daily summary complete for {}", date);

    Ok(())
}

/// Find all hourly reports for a given date
fn find_hourly_reports(report_dir: &Path, date: &str) -> Result<Vec<PathBuf>, DailyError> {
    let mut reports = Vec::new();

    let entries = fs::read_dir(report_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            // Match pattern: YYYY-MM-DD-HH.md
            if filename.starts_with(date) && filename.ends_with(".md") && filename.len() == 16 {
                // Verify it's an hourly report (has -HH before .md)
                if filename.chars().nth(10) == Some('-') && filename.chars().nth(13) == Some('.') {
                    reports.push(path);
                }
            }
        }
    }

    // Sort by filename (chronological order)
    reports.sort();

    Ok(reports)
}

/// Build prompt for daily summary
fn build_daily_prompt(hourly_reports: &[String], date: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "You are Marvin, reviewing the entire day's worth of hourly monitoring reports for {}.\n\n",
        date
    ));

    prompt.push_str("TASK: Create a concise daily summary that highlights:\n");
    prompt.push_str("- Overall system health trend for the day\n");
    prompt.push_str("- Any recurring issues or patterns\n");
    prompt.push_str("- Notable events worth remembering\n");
    prompt.push_str("- Critical or concerning issues (if any)\n");
    prompt.push_str("- Temperature/sensor trends across the day\n\n");

    prompt.push_str("Keep it brief - this is a daily digest, not a novel.\n");
    prompt.push_str("Use your characteristic depressed tone but be clear about any real problems.\n\n");

    prompt.push_str("OUTPUT FORMAT:\n");
    prompt.push_str("# Marvinous Daily Summary: [DATE]\n\n");
    prompt.push_str("## Day Overview\n");
    prompt.push_str("[SEVERITY]: [One sentence summary]\n\n");
    prompt.push_str("## Key Events\n");
    prompt.push_str("[Bullet points of notable occurrences]\n\n");
    prompt.push_str("## System Health\n");
    prompt.push_str("[Brief assessment of overall health]\n\n");
    prompt.push_str("## Trends\n");
    prompt.push_str("[Any patterns observed across the day]\n\n");

    prompt.push_str(&format!("=== HOURLY REPORTS FOR {} ===\n\n", date));

    for (i, report) in hourly_reports.iter().enumerate() {
        prompt.push_str(&format!("--- Hour {} ---\n", i));
        prompt.push_str(report);
        prompt.push_str("\n\n");
    }

    prompt
}

/// Write daily summary to file
fn write_daily_summary(
    report_dir: &Path,
    date: &str,
    summary: &str,
) -> Result<PathBuf, DailyError> {
    let filename = format!("{}-DAILY.md", date);
    let path = report_dir.join(filename);

    let mut file = File::create(&path)?;
    file.write_all(summary.as_bytes())?;

    Ok(path)
}

/// Archive hourly reports to a zip file
fn archive_hourly_reports(
    report_dir: &Path,
    date: &str,
    reports: &[PathBuf],
) -> Result<PathBuf, DailyError> {
    let archive_dir = report_dir.join("archive");
    fs::create_dir_all(&archive_dir)?;

    let archive_filename = format!("{}.zip", date);
    let archive_path = archive_dir.join(archive_filename);

    let file = File::create(&archive_path)
        .map_err(|e| DailyError::ArchiveError(e.to_string()))?;

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for report_path in reports {
        if let Some(filename) = report_path.file_name().and_then(|s| s.to_str()) {
            let content = fs::read(report_path)?;

            zip.start_file(filename, options)
                .map_err(|e| DailyError::ArchiveError(e.to_string()))?;

            zip.write_all(&content)
                .map_err(|e| DailyError::ArchiveError(e.to_string()))?;
        }
    }

    zip.finish()
        .map_err(|e| DailyError::ArchiveError(e.to_string()))?;

    Ok(archive_path)
}
