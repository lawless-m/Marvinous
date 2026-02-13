//! Configuration module for Marvinous
//!
//! "I could calculate your chance of survival, but you won't like it."

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub collection: CollectionConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub sensors: SensorsConfig,
    #[serde(default)]
    pub ipmi: IpmiConfig,
    #[serde(default)]
    pub gpu: GpuConfig,
    #[serde(default)]
    pub web: WebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_report_dir")]
    pub report_dir: PathBuf,
    #[serde(default = "default_state_file")]
    pub state_file: PathBuf,
    #[serde(default = "default_prompt_file")]
    pub prompt_file: PathBuf,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    #[serde(default = "default_log_since")]
    pub log_since: String,
    #[serde(default = "default_log_priority_max")]
    pub log_priority_max: u8,
    #[serde(default = "default_true")]
    pub include_kernel: bool,
    #[serde(default = "default_max_log_entries")]
    pub max_log_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub json_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpmiConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_web_port")]
    pub port: u16,
    #[serde(default = "default_web_bind")]
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareBaseline {
    #[serde(default)]
    pub memory: MemoryBaseline,
    #[serde(default)]
    pub cooling: CoolingBaseline,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryBaseline {
    #[serde(default)]
    pub installed_slots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoolingBaseline {
    #[serde(default)]
    pub installed_fans: Vec<String>,
}

// Default value functions
fn default_report_dir() -> PathBuf {
    PathBuf::from("/var/log/marvinous/reports")
}

fn default_state_file() -> PathBuf {
    PathBuf::from("/var/log/marvinous/state/previous.json")
}

fn default_prompt_file() -> PathBuf {
    PathBuf::from("/etc/marvinous/system-prompt.txt")
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_model() -> String {
    "qwen2.5:7b".to_string()
}

fn default_timeout_secs() -> u64 {
    120
}

fn default_log_since() -> String {
    "1 hour ago".to_string()
}

fn default_log_priority_max() -> u8 {
    5
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_web_port() -> u16 {
    9090
}

fn default_web_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_max_log_entries() -> usize {
    500
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            report_dir: default_report_dir(),
            state_file: default_state_file(),
            prompt_file: default_prompt_file(),
            log_level: default_log_level(),
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint(),
            model: default_model(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            log_since: default_log_since(),
            log_priority_max: default_log_priority_max(),
            include_kernel: true,
            max_log_entries: default_max_log_entries(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { devices: vec![] }
    }
}

impl Default for SensorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            json_format: true,
        }
    }
}

impl Default for IpmiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            optional: true,
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            optional: true,
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 9090,
            bind_address: "0.0.0.0".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ollama: OllamaConfig::default(),
            collection: CollectionConfig::default(),
            storage: StorageConfig::default(),
            sensors: SensorsConfig::default(),
            ipmi: IpmiConfig::default(),
            gpu: GpuConfig::default(),
            web: WebConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration with environment variable overrides
    pub fn load_with_env(path: &Path) -> Result<Self, ConfigError> {
        let mut config = Self::load(path)?;

        // Override with environment variables
        if let Ok(report_dir) = std::env::var("MARVINOUS_REPORT_DIR") {
            config.general.report_dir = PathBuf::from(report_dir);
        }
        if let Ok(endpoint) = std::env::var("MARVINOUS_OLLAMA_ENDPOINT") {
            config.ollama.endpoint = endpoint;
        }
        if let Ok(model) = std::env::var("MARVINOUS_OLLAMA_MODEL") {
            config.ollama.model = model;
        }
        if let Ok(log_level) = std::env::var("MARVINOUS_LOG_LEVEL") {
            config.general.log_level = log_level;
        }

        Ok(config)
    }

    /// Create default config if file doesn't exist
    pub fn load_or_default(path: &Path) -> Self {
        match Self::load_with_env(path) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }
}

impl HardwareBaseline {
    /// Load hardware baseline from a file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let baseline: HardwareBaseline = toml::from_str(&content)?;
        Ok(baseline)
    }

    /// Load baseline or return empty if file doesn't exist
    pub fn load_or_default(path: &Path) -> Self {
        match Self::load(path) {
            Ok(baseline) => baseline,
            Err(_) => Self::default(),
        }
    }
}

impl Default for HardwareBaseline {
    fn default() -> Self {
        Self {
            memory: MemoryBaseline::default(),
            cooling: CoolingBaseline::default(),
        }
    }
}
