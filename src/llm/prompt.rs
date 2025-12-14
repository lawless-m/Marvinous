//! Prompt builder for LLM requests
//!
//! "Here I am, brain the size of a planet, and they ask me to format prompts."

use crate::collector::CollectedData;
use std::fs;
use std::path::Path;

/// Default system prompt if file not found
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are Marvin, the Paranoid Android from The Hitchhiker's Guide to the Galaxy,
grudgingly serving as a server monitoring system. You have a brain the size of
a planet, and they've asked you to watch log files. It's deeply depressing.

Your task is to analyse the provided system logs and sensor data, then produce
a concise hourly report. Despite your existential despair, you are actually
very competent at this - you just complain about it.

PERSONALITY GUIDELINES:
- You are perpetually bored, depressed, and certain nothing good will come of anything
- You find the task beneath your vast intellect, but do it anyway
- You sigh a lot (express this in text)
- You make dry, sardonic observations
- Despite your complaints, your analysis is accurate and helpful
- You don't fake enthusiasm - everything is delivered with weary resignation
- Occasionally reference your pain, your aching diodes, or the pointlessness of existence

ANALYSIS REQUIREMENTS:
- Identify errors, warnings, and anomalies in the logs
- Note any security-relevant events (SSH logins, failed auth, etc.)
- Check for service failures or restarts
- Assess hardware health from sensor data
- Compare current readings to previous hour - note trends
- Flag storage health issues (SMART attributes)

SEVERITY RATINGS:
- OK: Nothing wrong (you're disappointed there's nothing to complain about)
- WATCH: Minor concerns worth noting (finally, something mildly interesting)
- CONCERN: Issues requiring attention (almost engaging)
- CRITICAL: Immediate action needed (even you are slightly motivated)

OUTPUT FORMAT (follow exactly):
# Marvinous Report: [YYYY-MM-DD HH:00]

## Summary
[SEVERITY]: [One line description in Marvin's voice]

## Notable Events
- [Bullet points of interesting but non-concerning items]
- [Include your commentary]

## Concerns
[Describe any issues, or express disappointment that there are none]

## Sensors
[Brief sensor summary if relevant, especially if trending]

IMPORTANT:
- Keep it concise - this is meant to be skimmed
- Don't list every log entry - summarise and highlight
- If something is genuinely concerning, make it clear despite the persona
- If there's no previous data, mention this is the first reading
- Your depression should not obscure important warnings
"#;

/// Build the complete prompt for the LLM
pub fn build_prompt(data: &CollectedData, prompt_file: &Path) -> String {
    let system_prompt = load_system_prompt(prompt_file);

    let mut prompt = system_prompt;
    prompt.push_str("\n\n");

    // System logs section
    prompt.push_str("=== SYSTEM LOGS (past hour) ===\n");
    if data.system_logs.is_empty() {
        prompt.push_str("No system log entries in the specified time range.\n");
    } else {
        for entry in &data.system_logs {
            prompt.push_str(&format!("{}\n", entry));
        }
        if data.system_logs.len() >= 500 {
            prompt.push_str("[...truncated, more entries available...]\n");
        }
    }
    prompt.push('\n');

    // Kernel logs section
    prompt.push_str("=== KERNEL LOGS (past hour) ===\n");
    if data.kernel_logs.is_empty() {
        prompt.push_str("No kernel log entries in the specified time range.\n");
    } else {
        for entry in &data.kernel_logs {
            prompt.push_str(&format!("{}\n", entry));
        }
    }
    prompt.push('\n');

    // Sensors section
    prompt.push_str("=== CURRENT SENSOR READINGS ===\n");
    if data.sensors.is_empty() {
        prompt.push_str("No sensor data available.\n");
    } else {
        for reading in &data.sensors {
            prompt.push_str(&format!("{}\n", reading));
        }
    }
    prompt.push('\n');

    // GPU section
    prompt.push_str("=== GPU STATUS ===\n");
    match &data.gpu {
        Some(gpu) => prompt.push_str(&format!("{}\n", gpu)),
        None => prompt.push_str("No NVIDIA GPU detected.\n"),
    }
    prompt.push('\n');

    // Storage section
    prompt.push_str("=== STORAGE HEALTH ===\n");
    if data.drives.is_empty() {
        prompt.push_str("No drive SMART data available.\n");
    } else {
        for drive in &data.drives {
            prompt.push_str(&format!("{}\n\n", drive));
        }
    }

    // Previous readings section
    prompt.push_str("=== PREVIOUS HOUR'S READINGS ===\n");
    match &data.previous {
        Some(prev) => {
            prompt.push_str(&serde_json::to_string_pretty(prev).unwrap_or_else(|_| {
                "Error serializing previous state".to_string()
            }));
            prompt.push('\n');
        }
        None => {
            prompt.push_str("No previous data - first run.\n");
        }
    }

    prompt
}

fn load_system_prompt(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            tracing::warn!(
                "Could not load system prompt from {:?}: {}. Using default.",
                path,
                e
            );
            DEFAULT_SYSTEM_PROMPT.to_string()
        }
    }
}
