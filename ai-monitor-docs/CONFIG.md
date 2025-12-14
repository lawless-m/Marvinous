# Marvinous - Configuration Reference

*"I could calculate your chance of survival, but you won't like it."*

## Configuration File

Default location: `/etc/marvinous/marvinous.toml`

Override with: `marvinous --config /path/to/config.toml`

## Full Configuration Example

```toml
# Marvinous Configuration
# "I've got this terrible pain in all the diodes down my left side."

[general]
# Where to write hourly reports
report_dir = "/var/log/marvinous/reports"

# State file for trend comparison
state_file = "/var/log/marvinous/state/previous.json"

# System prompt template
prompt_file = "/etc/marvinous/system-prompt.txt"

# Log level: trace, debug, info, warn, error
log_level = "info"


[ollama]
# Ollama API endpoint
endpoint = "http://localhost:11434"

# Model to use - Qwen 2.5 recommended
model = "qwen2.5:7b"

# Timeout for LLM generation (seconds)
timeout_secs = 120


[collection]
# How far back to collect logs
log_since = "1 hour ago"

# Maximum log priority (0=emerg, 5=notice, 7=debug)
# Default 5 includes: emerg, alert, crit, err, warning, notice
# Excludes: info (6), debug (7)
log_priority_max = 5

# Include kernel ring buffer logs
include_kernel = true

# Maximum log entries to send to LLM (prevents token overflow)
max_log_entries = 500


[storage]
# Drives to monitor with smartctl
# Empty array = auto-detect all drives
# Explicit list = only these drives
devices = []

# Examples:
# devices = ["/dev/sda", "/dev/sdb"]
# devices = ["/dev/nvme0n1", "/dev/nvme1n1"]


[sensors]
# Include sensors output
enabled = true

# Use JSON output (recommended)
json_format = true


[gpu]
# Include nvidia-smi output
enabled = true

# Fail silently if no NVIDIA GPU present
optional = true


[notifications]
# Future feature - not yet implemented
enabled = false

# Notification endpoint (ntfy, webhook, etc.)
# endpoint = "https://ntfy.sh/my-server-alerts"

# Only notify on these severities
# notify_on = ["CONCERN", "CRITICAL"]
```

## Configuration Sections

### `[general]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `report_dir` | String | `/var/log/marvinous/reports` | Directory for markdown reports |
| `state_file` | String | `/var/log/marvinous/state/previous.json` | Previous readings for trend comparison |
| `prompt_file` | String | `/etc/marvinous/system-prompt.txt` | LLM system prompt template |
| `log_level` | String | `info` | Logging verbosity |

### `[ollama]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `endpoint` | String | `http://localhost:11434` | Ollama API URL |
| `model` | String | `qwen2.5:7b` | Model name |
| `timeout_secs` | Integer | `120` | HTTP timeout |

### `[collection]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `log_since` | String | `1 hour ago` | journalctl time range |
| `log_priority_max` | Integer | `5` | Maximum priority level |
| `include_kernel` | Boolean | `true` | Include `-k` kernel logs |
| `max_log_entries` | Integer | `500` | Truncate at this many entries |

### `[storage]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `devices` | Array | `[]` | Drives to check (empty = auto-detect) |

### `[sensors]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | Boolean | `true` | Collect sensor data |
| `json_format` | Boolean | `true` | Use `sensors -j` |

### `[gpu]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | Boolean | `true` | Collect GPU data |
| `optional` | Boolean | `true` | Don't fail if no GPU |

### `[notifications]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | Boolean | `false` | Enable notifications |

## Environment Variables

Configuration can be overridden with environment variables:

| Variable | Overrides |
|----------|-----------|
| `MARVINOUS_CONFIG` | Config file path |
| `MARVINOUS_REPORT_DIR` | `general.report_dir` |
| `MARVINOUS_OLLAMA_ENDPOINT` | `ollama.endpoint` |
| `MARVINOUS_OLLAMA_MODEL` | `ollama.model` |
| `MARVINOUS_LOG_LEVEL` | `general.log_level` |

Environment variables take precedence over config file values.

## Directory Setup

Marvinous expects these directories to exist:

```bash
sudo mkdir -p /etc/marvinous
sudo mkdir -p /var/log/marvinous/reports
sudo mkdir -p /var/log/marvinous/state
```

## Minimal Configuration

For most systems, the defaults work. A minimal config:

```toml
# /etc/marvinous/marvinous.toml

[ollama]
model = "qwen2.5:7b"
```

Everything else uses sensible defaults.

## Model Selection

Tested models:

| Model | VRAM Required | Notes |
|-------|---------------|-------|
| `qwen2.5:7b` | ~6GB | Recommended, good balance |
| `qwen2.5:14b` | ~12GB | Better analysis, slower |
| `qwen2.5:32b` | ~24GB | Fits your 3090, overkill |
| `llama3.1:8b` | ~6GB | Alternative, slightly less structured |
| `mistral:7b` | ~6GB | Alternative, works well |

For your 3090 with 24GB VRAM, `qwen2.5:7b` runs very fast with plenty of headroom.

---

*"Funny, how just when you think life can't possibly get any worse it suddenly does."*
