//! Marvinous - Server monitoring with the personality of a paranoid android
//!
//! "Here I am, brain the size of a planet, and they ask me to watch log files."

mod collector;
mod config;
mod llm;
mod output;

use chrono::Utc;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

use collector::{
    collect_gpu, collect_ipmi, collect_kernel_logs, collect_sensors, collect_smart,
    collect_system_logs, CollectedData,
};
use config::Config;
use llm::{build_prompt, OllamaClient};
use output::{load_previous, parse_severity, save_current, write_report, PreviousState};

/// Marvinous - Server monitoring with existential despair
#[derive(Parser, Debug)]
#[command(name = "marvinous")]
#[command(about = "Server monitoring with the personality of a paranoid android")]
#[command(version)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "/etc/marvinous/marvinous.toml")]
    config: PathBuf,

    /// Print collected data without calling LLM
    #[arg(long)]
    dry_run: bool,

    /// Print the prompt that would be sent to the LLM
    #[arg(long)]
    show_prompt: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    // Load configuration
    let config = Config::load_or_default(&args.config);

    // Initialize logging
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.general.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();

    tracing::info!("Marvinous starting - *sigh* here we go again");

    // Run the main logic and handle exit codes
    match run(&config, &args).await {
        Ok(_) => {
            tracing::info!("Complete. Not that it matters.");
            ExitCode::from(0)
        }
        Err(e) => {
            tracing::error!("Failed: {}", e);
            e.exit_code()
        }
    }
}

#[derive(Debug)]
enum MarvinError {
    #[allow(dead_code)]
    Config(String),
    Collection(String),
    Ollama(String),
    Write(String),
}

impl MarvinError {
    fn exit_code(&self) -> ExitCode {
        match self {
            MarvinError::Config(_) => ExitCode::from(1),
            MarvinError::Collection(_) => ExitCode::from(2),
            MarvinError::Ollama(_) => ExitCode::from(3),
            MarvinError::Write(_) => ExitCode::from(4),
        }
    }
}

impl std::fmt::Display for MarvinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarvinError::Config(msg) => write!(f, "Configuration error: {}", msg),
            MarvinError::Collection(msg) => write!(f, "Collection error: {}", msg),
            MarvinError::Ollama(msg) => write!(f, "Ollama error: {}", msg),
            MarvinError::Write(msg) => write!(f, "Write error: {}", msg),
        }
    }
}

async fn run(config: &Config, args: &Args) -> Result<(), MarvinError> {
    // Collect data
    tracing::info!("Starting collection");

    let system_logs = match collect_system_logs(
        &config.collection.log_since,
        config.collection.log_priority_max,
        config.collection.max_log_entries,
    ) {
        Ok(logs) => {
            tracing::info!("Collected {} system log entries", logs.len());
            logs
        }
        Err(e) => {
            tracing::warn!("Failed to collect system logs: {}", e);
            vec![]
        }
    };

    let kernel_logs = if config.collection.include_kernel {
        match collect_kernel_logs(&config.collection.log_since, config.collection.max_log_entries) {
            Ok(logs) => {
                tracing::info!("Collected {} kernel log entries", logs.len());
                logs
            }
            Err(e) => {
                tracing::warn!("Failed to collect kernel logs: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    let sensors = if config.sensors.enabled {
        match collect_sensors() {
            Ok(readings) => {
                tracing::info!("Collected {} sensor readings", readings.len());
                readings
            }
            Err(e) => {
                tracing::warn!("Failed to collect sensor data: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    let ipmi = if config.ipmi.enabled {
        match collect_ipmi() {
            Ok(readings) => {
                tracing::info!("Collected {} IPMI sensor readings", readings.len());
                readings
            }
            Err(e) => {
                if config.ipmi.optional {
                    tracing::warn!("Failed to collect IPMI data (optional): {}", e);
                    vec![]
                } else {
                    return Err(MarvinError::Collection(format!("IPMI collection failed: {}", e)));
                }
            }
        }
    } else {
        vec![]
    };

    let gpu = if config.gpu.enabled {
        match collect_gpu() {
            Ok(Some(gpu)) => {
                tracing::info!("GPU detected: {}", gpu.name);
                Some(gpu)
            }
            Ok(None) => {
                tracing::info!("No NVIDIA GPU detected");
                None
            }
            Err(e) => {
                if config.gpu.optional {
                    tracing::warn!("Failed to collect GPU data (optional): {}", e);
                    None
                } else {
                    return Err(MarvinError::Collection(format!("GPU collection failed: {}", e)));
                }
            }
        }
    } else {
        None
    };

    let drives = match collect_smart(&config.storage.devices) {
        Ok(drives) => {
            tracing::info!("Checked {} drives", drives.len());
            drives
        }
        Err(e) => {
            tracing::warn!("Failed to collect SMART data: {}", e);
            vec![]
        }
    };

    // Load previous state
    let previous = match load_previous(&config.general.state_file) {
        Ok(prev) => prev,
        Err(e) => {
            tracing::warn!("Failed to load previous state: {}", e);
            None
        }
    };

    let collected = CollectedData {
        system_logs,
        kernel_logs,
        sensors: sensors.clone(),
        ipmi: ipmi.clone(),
        gpu: gpu.clone(),
        drives: drives.clone(),
        previous,
    };

    // Dry run mode
    if args.dry_run {
        println!("=== Collected Data ===");
        println!(
            "{}",
            serde_json::to_string_pretty(&collected).unwrap_or_else(|_| "Error".to_string())
        );
        return Ok(());
    }

    // Build prompt
    let prompt = build_prompt(&collected, &config.general.prompt_file);

    // Show prompt mode
    if args.show_prompt {
        println!("=== Prompt ===");
        println!("{}", prompt);
        return Ok(());
    }

    // Initialize Ollama client
    let client = OllamaClient::new(
        &config.ollama.endpoint,
        &config.ollama.model,
        config.ollama.timeout_secs,
    );

    // Health check
    if let Err(e) = client.health_check().await {
        return Err(MarvinError::Ollama(format!(
            "Ollama not reachable at {}: {}",
            config.ollama.endpoint, e
        )));
    }

    tracing::info!(
        "Sending prompt to Ollama ({} chars)",
        prompt.len()
    );

    // Generate report
    let report = match client.generate(&prompt).await {
        Ok(response) => {
            tracing::info!("Response received ({} chars)", response.len());
            response
        }
        Err(e) => {
            return Err(MarvinError::Ollama(format!("Generation failed: {}", e)));
        }
    };

    // Parse severity
    let severity = parse_severity(&report);
    tracing::info!("Report severity: {}", severity);

    // Write report
    let timestamp = Utc::now();
    let report_path = write_report(&config.general.report_dir, timestamp, &report)
        .map_err(|e| MarvinError::Write(e.to_string()))?;

    println!("Report written to: {}", report_path.display());
    println!("Severity: {}", severity);

    // Save current state for next run
    let current_state = PreviousState::new(sensors, gpu, drives);
    if let Err(e) = save_current(&config.general.state_file, &current_state) {
        tracing::warn!("Failed to save state: {}", e);
    }

    Ok(())
}
