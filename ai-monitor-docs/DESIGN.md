# Marvinous - Design Document

*"I have a million ideas, but they all point to certain death."*

## Overview

Marvinous runs hourly via systemd timer, collects system data, sends it to a local LLM for analysis, and writes a markdown report. Simple pipeline, minimal moving parts.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Systemd Timer                           │
│                   (hourly trigger)                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                      Collectors                             │
├─────────────┬─────────────┬──────────────┬─────────────────┤
│ journalctl  │   sensors   │  nvidia-smi  │    smartctl     │
│  (logs)     │  (thermal)  │    (GPU)     │   (storage)     │
└──────┬──────┴──────┬──────┴───────┬──────┴────────┬────────┘
       │             │              │               │
       └─────────────┴──────────────┴───────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    State Manager                            │
│         (load previous.json, prepare comparison)            │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Prompt Builder                            │
│    (assemble data + system prompt + Marvin instructions)    │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Ollama Client                            │
│              (Qwen 2.5 via HTTP API)                        │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Output Handler                            │
├─────────────────────────┬───────────────────────────────────┤
│  Write report.md        │  Update previous.json             │
│  (Maybe notify)         │  (sensor snapshot)                │
└─────────────────────────┴───────────────────────────────────┘
```

## Component Design

### 1. Collectors (`src/collector/`)

Each collector is a module that knows how to gather one type of data:

#### `journalctl.rs`
```rust
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub priority: u8,
    pub unit: Option<String>,
    pub message: String,
}

pub fn collect_system_logs(since: Duration) -> Result<Vec<LogEntry>>;
pub fn collect_kernel_logs(since: Duration) -> Result<Vec<LogEntry>>;
```

Executes:
- `journalctl --since "1 hour ago" --priority=0..5 --output=json`
- `journalctl -k --since "1 hour ago" --output=json`

#### `sensors.rs`
```rust
pub struct SensorReading {
    pub chip: String,
    pub sensor: String,
    pub value: f64,
    pub unit: String,  // "°C", "RPM", "V"
}

pub fn collect_sensors() -> Result<Vec<SensorReading>>;
```

Executes: `sensors -j` (JSON output)

#### `nvidia.rs`
```rust
pub struct GpuStatus {
    pub name: String,
    pub temperature: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub utilisation: u8,
    pub power_draw: f64,
}

pub fn collect_gpu() -> Result<Option<GpuStatus>>;
```

Executes: `nvidia-smi --query-gpu=name,temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw --format=csv,noheader,nounits`

Returns `None` if no NVIDIA GPU present.

#### `smart.rs`
```rust
pub struct DriveHealth {
    pub device: String,
    pub model: String,
    pub reallocated_sectors: u64,
    pub pending_sectors: u64,
    pub temperature: Option<f64>,
    pub power_on_hours: u64,
}

pub fn collect_smart(devices: &[String]) -> Result<Vec<DriveHealth>>;
```

Executes: `smartctl -A /dev/sdX --json` for each device

### 2. State Manager (`src/output/state.rs`)

```rust
pub struct PreviousState {
    pub timestamp: DateTime<Utc>,
    pub sensors: Vec<SensorReading>,
    pub gpu: Option<GpuStatus>,
    pub drives: Vec<DriveHealth>,
}

pub fn load_previous(path: &Path) -> Result<Option<PreviousState>>;
pub fn save_current(path: &Path, state: &PreviousState) -> Result<()>;
```

Simple JSON serialisation. First run has no previous state - LLM handles this gracefully.

### 3. Prompt Builder (`src/llm/prompt.rs`)

```rust
pub struct CollectedData {
    pub system_logs: Vec<LogEntry>,
    pub kernel_logs: Vec<LogEntry>,
    pub sensors: Vec<SensorReading>,
    pub gpu: Option<GpuStatus>,
    pub drives: Vec<DriveHealth>,
    pub previous: Option<PreviousState>,
}

pub fn build_prompt(data: &CollectedData, template: &str) -> String;
```

Assembles the full prompt from template + collected data. See `PROMPT.md` for the template.

### 4. Ollama Client (`src/llm/ollama.rs`)

```rust
pub struct OllamaClient {
    endpoint: String,
    model: String,
}

impl OllamaClient {
    pub fn new(endpoint: &str, model: &str) -> Self;
    pub async fn generate(&self, prompt: &str) -> Result<String>;
}
```

Simple HTTP POST to Ollama's `/api/generate` endpoint. No streaming needed - we want the complete response.

### 5. Output Handler (`src/output/report.rs`)

```rust
pub fn write_report(
    report_dir: &Path,
    timestamp: DateTime<Utc>,
    content: &str,
) -> Result<PathBuf>;

pub fn parse_severity(content: &str) -> Severity;
```

Writes `YYYY-MM-DD-HH.md` to the reports directory. Optionally parses the severity from the report for future notification logic.

## Data Flow (Detailed)

1. **Timer fires** → systemd starts `marvinous.service`

2. **Load config** → Read `/etc/marvinous/marvinous.toml`

3. **Collect data** (parallel where sensible):
   - journalctl system logs
   - journalctl kernel logs  
   - sensors readings
   - nvidia-smi (if present)
   - smartctl for configured drives

4. **Load previous state** → Read `/var/log/marvinous/state/previous.json`

5. **Build prompt** → Combine system prompt template + all collected data + previous readings

6. **Call Ollama** → POST to local Qwen instance, receive Marvin's report

7. **Write outputs**:
   - Report → `/var/log/marvinous/reports/2025-12-14-15.md`
   - State → `/var/log/marvinous/state/previous.json` (current sensor values)

8. **Exit** → Process terminates until next timer tick

## Error Handling

- **Collector failure**: Log warning, continue with available data. LLM can note "GPU data unavailable" etc.
- **Ollama unreachable**: Write error report, exit non-zero for systemd to log
- **Previous state missing**: Normal on first run, prompt indicates "no previous data"
- **Disk full**: Fail hard, exit non-zero

## Token Budget Considerations

Qwen 2.5 7B has 128k context, but we should still be sensible:

- **System logs**: Could be thousands of lines. Pre-filter to unique messages, cap at ~500 entries
- **Kernel logs**: Usually sparse, include all
- **Sensors**: Compact, no issue
- **SMART**: Compact, no issue
- **Previous state**: Compact, no issue

If a busy hour produces excessive logs, truncate with a note: `[...truncated, 2847 additional entries...]`

## Future Considerations (Not in Initial Scope)

- **Notifications**: Send alert on CONCERN/CRITICAL (ntfy, email, etc.)
- **Web viewer**: Simple static site to browse reports
- **Retention**: Auto-delete reports older than N days
- **Multiple hosts**: Aggregate reports from fleet

---

*"Life. Don't talk to me about life."*
