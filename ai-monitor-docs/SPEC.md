# Marvinous - Technical Specification

*"I'd make a suggestion, but you wouldn't listen. No one ever does."*

## External Dependencies

### System Commands

| Command | Package | Purpose | Required |
|---------|---------|---------|----------|
| `journalctl` | systemd | Log collection | Yes |
| `sensors` | lm-sensors | Thermal/voltage readings | Yes |
| `nvidia-smi` | nvidia-driver | GPU monitoring | No |
| `smartctl` | smartmontools | Drive health | Yes |

### Services

| Service | Default Endpoint | Purpose |
|---------|------------------|---------|
| Ollama | `http://localhost:11434` | LLM inference |

### Rust Crates (Suggested)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
```

## Command Invocations

### journalctl - System Logs

```bash
journalctl --since "1 hour ago" --priority=0..5 --output=json --no-pager
```

Output (one JSON object per line):
```json
{
  "__REALTIME_TIMESTAMP": "1702569600000000",
  "PRIORITY": "4",
  "_SYSTEMD_UNIT": "ssh.service",
  "MESSAGE": "Accepted publickey for dave from 192.168.1.50"
}
```

### journalctl - Kernel Logs

```bash
journalctl -k --since "1 hour ago" --output=json --no-pager
```

Same format as system logs, but `_TRANSPORT=kernel`.

### sensors

```bash
sensors -j
```

Output:
```json
{
  "coretemp-isa-0000": {
    "Adapter": "ISA adapter",
    "Core 0": {
      "temp2_input": 45.000,
      "temp2_max": 100.000,
      "temp2_crit": 100.000
    }
  }
}
```

### nvidia-smi

```bash
nvidia-smi --query-gpu=name,temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw --format=csv,noheader,nounits
```

Output:
```
NVIDIA GeForce RTX 3090, 42, 1234, 24576, 15, 120.5
```

### smartctl

```bash
smartctl -A /dev/sda --json
```

Output (excerpt):
```json
{
  "ata_smart_attributes": {
    "table": [
      {
        "id": 5,
        "name": "Reallocated_Sector_Ct",
        "raw": { "value": 0 }
      },
      {
        "id": 197,
        "name": "Current_Pending_Sector",
        "raw": { "value": 0 }
      }
    ]
  }
}
```

Key SMART attributes to track:
- ID 5: Reallocated_Sector_Ct
- ID 197: Current_Pending_Sector  
- ID 194: Temperature_Celsius
- ID 9: Power_On_Hours

## Ollama API

### Generate Endpoint

```
POST http://localhost:11434/api/generate
Content-Type: application/json

{
  "model": "qwen2.5:7b",
  "prompt": "...",
  "stream": false
}
```

Response:
```json
{
  "model": "qwen2.5:7b",
  "response": "# Marvinous Report: 2025-12-14 15:00\n\n...",
  "done": true
}
```

### Timeout

Set HTTP timeout to 120 seconds. LLM generation can take a while on large inputs.

## File Formats

### Configuration: `marvinous.toml`

```toml
[general]
report_dir = "/var/log/marvinous/reports"
state_file = "/var/log/marvinous/state/previous.json"
prompt_file = "/etc/marvinous/system-prompt.txt"

[ollama]
endpoint = "http://localhost:11434"
model = "qwen2.5:7b"
timeout_secs = 120

[collection]
log_since = "1 hour ago"
log_priority_max = 5
include_kernel = true
max_log_entries = 500

[storage]
# Empty = auto-detect, or list specific devices
devices = []
# devices = ["/dev/sda", "/dev/sdb"]

[notifications]
# Future feature
enabled = false
```

### State: `previous.json`

```json
{
  "timestamp": "2025-12-14T14:00:00Z",
  "sensors": [
    {
      "chip": "coretemp-isa-0000",
      "sensor": "Core 0",
      "value": 45.0,
      "unit": "°C"
    }
  ],
  "gpu": {
    "name": "NVIDIA GeForce RTX 3090",
    "temperature": 42.0,
    "memory_used": 1234,
    "memory_total": 24576,
    "utilisation": 15,
    "power_draw": 120.5
  },
  "drives": [
    {
      "device": "/dev/sda",
      "model": "Samsung SSD 970 EVO",
      "reallocated_sectors": 0,
      "pending_sectors": 0,
      "temperature": 35.0,
      "power_on_hours": 12345
    }
  ]
}
```

### Report: `YYYY-MM-DD-HH.md`

See `EXAMPLE-OUTPUT.md` for full examples. Filename format:
- `2025-12-14-15.md` = Report for 15:00 on 2025-12-14

## Systemd Units

### `marvinous.service`

```ini
[Unit]
Description=Marvinous Server Monitor
After=network.target ollama.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/marvinous
User=root
Group=root

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=marvinous

[Install]
WantedBy=multi-user.target
```

### `marvinous.timer`

```ini
[Unit]
Description=Run Marvinous hourly

[Timer]
OnCalendar=hourly
Persistent=true

[Install]
WantedBy=timers.target
```

## Directory Structure

### Installation

```
/usr/local/bin/marvinous          # Binary
/etc/marvinous/
├── marvinous.toml                # Configuration
└── system-prompt.txt             # LLM prompt template
```

### Runtime

```
/var/log/marvinous/
├── reports/
│   ├── 2025-12-14-13.md
│   ├── 2025-12-14-14.md
│   └── 2025-12-14-15.md
└── state/
    └── previous.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Configuration error |
| 2 | Collection error (partial data collected) |
| 3 | Ollama unreachable |
| 4 | Write error (disk full, permissions) |

## Permissions

- Runs as root (required for smartctl, full journalctl access)
- Creates directories with 0755
- Creates files with 0644
- State file contains no secrets

## Logging

Uses `tracing` crate. Default level: INFO.

```
2025-12-14T15:00:01Z INFO marvinous: Starting collection
2025-12-14T15:00:01Z INFO marvinous::collector::journalctl: Collected 127 system log entries
2025-12-14T15:00:01Z INFO marvinous::collector::journalctl: Collected 12 kernel log entries
2025-12-14T15:00:02Z INFO marvinous::collector::sensors: Collected 8 sensor readings
2025-12-14T15:00:02Z INFO marvinous::collector::nvidia: GPU detected: RTX 3090
2025-12-14T15:00:03Z INFO marvinous::collector::smart: Checked 2 drives
2025-12-14T15:00:03Z INFO marvinous::llm: Sending prompt to Ollama (12847 tokens estimated)
2025-12-14T15:00:15Z INFO marvinous::llm: Response received (847 tokens)
2025-12-14T15:00:15Z INFO marvinous::output: Report written: /var/log/marvinous/reports/2025-12-14-15.md
2025-12-14T15:00:15Z INFO marvinous: Complete
```

---

*"I've seen it. It's rubbish."*
