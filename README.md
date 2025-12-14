# Marvinous

![Marvinous](Marvinous.png)

*"Here I am, brain the size of a planet, and they ask me to monitor server metrics. Call that job satisfaction? 'Cause I don't."*

An LLM-powered server monitoring tool with the personality of Marvin the Paranoid Android from The Hitchhiker's Guide to the Galaxy.

## Overview

Marvinous watches your servers with the same enthusiasm Marvin has for... well, everything. It collects comprehensive hardware metrics via IPMI, monitors storage health, and generates hourly reports with existential dread and sardonic observations.

**Features:**

- **IPMI BMC Monitoring** - 80+ sensors including CPU/DIMM temps, fan speeds, voltages, and current draw
- **SMART Drive Health** - Automated monitoring of reallocated sectors, pending sectors, and drive temperatures
- **GPU Monitoring** - NVIDIA GPU temperature, memory usage, and power draw tracking
- **LLM-Powered Reports** - Hourly analysis via Ollama (qwen2.5:7b) with Marvin's personality
- **Trend Analysis** - Compares current readings against previous hour to detect changes
- **Systemd Integration** - Automated hourly reports via systemd timer

## Quick Start

See [BUILD_AND_DEPLOY.md](BUILD_AND_DEPLOY.md) for complete installation instructions.

**Prerequisites:**
- Rust 1.70+
- Ollama with qwen2.5:7b model
- IPMI-capable server hardware (optional but recommended)
- ipmitool, smartmontools, lm-sensors

**Quick Install:**
```bash
# Install dependencies
sudo apt install build-essential pkg-config libssl-dev lm-sensors smartmontools ipmitool

# Build and install
cargo build --release
sudo cp target/release/marvinous /usr/local/bin/
sudo cp config/marvinous.toml /etc/marvinous/
sudo cp prompts/system-prompt.txt /etc/marvinous/
sudo cp systemd/marvinous.{service,timer} /etc/systemd/system/
sudo systemctl enable --now marvinous.timer
```

## Configuration

Edit `/etc/marvinous/marvinous.toml`:

```toml
[general]
report_dir = "/var/log/marvinous/reports"
state_file = "/var/log/marvinous/state/previous.json"
prompt_file = "/etc/marvinous/system-prompt.txt"

[ollama]
endpoint = "http://localhost:11434"
model = "qwen2.5:7b"

[ipmi]
enabled = true  # Comprehensive BMC sensor monitoring
optional = true

[gpu]
enabled = true  # NVIDIA GPU monitoring
optional = true
```

Update the hardware baseline in `/etc/marvinous/system-prompt.txt` to match your actual server configuration (CPUs, installed DIMMs, fans, drives).

## Usage

```bash
# Run manually (generates report immediately)
sudo marvinous

# View raw collected data without LLM analysis
sudo marvinous --dry-run

# See what prompt is sent to the LLM
sudo marvinous --show-prompt

# Check timer status
systemctl list-timers marvinous.timer

# View latest report
cat /var/log/marvinous/reports/$(ls -t /var/log/marvinous/reports/ | head -1)
```

## Sample Report

```markdown
# Marvinous Report: 2025-12-14 18:00

## Summary
OK: Today's readings are... just as boring as always.

## Notable Events
- Root login via sudo - what a waste of time.

## Concerns
Disappointingly, there are no issues to report.

## Sensors

### Fans
CPU0_FAN | 1900 RPM | ok
CPU1_FAN | 1900 RPM | ok
SYS_FAN5 | 1200 RPM | ok

### IPMI Sensors
- CPU0: 30°C, CPU1: 35°C
- DIMMs: 27-34°C (8 installed, 8 empty slots)
- All voltages nominal (12V, 5V, 3.3V rails)

### Storage Health
All drives healthy:
- Reallocated Sectors: 0
- Pending Sectors: 0
- Temperatures: 40°C

### GPU Status
NVIDIA GeForce RTX 3090 chilling at 46°C with barely any load.

*Signed off by Marvinous.*
```

## Architecture

```
marvinous/
├── src/
│   ├── main.rs              # Entry point and orchestration
│   ├── config.rs            # Configuration management
│   ├── collector/           # Data collection modules
│   │   ├── ipmi.rs          # IPMI BMC sensor collection
│   │   ├── smart.rs         # SMART drive health
│   │   ├── nvidia.rs        # GPU monitoring
│   │   ├── sensors.rs       # lm-sensors (optional)
│   │   └── journalctl.rs    # System/kernel logs
│   ├── llm/                 # LLM interaction
│   │   ├── client.rs        # Ollama API client
│   │   └── prompt.rs        # Prompt building
│   └── output/              # Report generation
│       ├── report.rs        # Markdown report writer
│       └── state.rs         # Trend comparison state
├── config/
│   ├── marvinous.toml       # Main configuration
│   └── hardware-baseline.toml  # Hardware documentation
├── prompts/
│   └── system-prompt.txt    # Marvin's personality & instructions
├── systemd/
│   ├── marvinous.service    # Systemd service unit
│   └── marvinous.timer      # Hourly execution timer
└── BUILD_AND_DEPLOY.md      # Deployment guide
```

## Monitored Metrics

### IPMI BMC (80 sensors)
- **Temperatures**: CPU packages, DIMM modules, voltage regulators, PCH
- **Fan Speeds**: CPU fans, chassis fans (RPM)
- **Voltages**: 12V, 5V, 3.3V rails and all CPU/DIMM power supplies
- **Current**: Per-CPU and per-DIMM bank amperage

### Storage (SMART)
- Device model and serial number
- Reallocated sectors (ID 5)
- Pending sectors (ID 197)
- Temperature (ID 194)
- Power-on hours (ID 9)

### GPU (nvidia-smi)
- GPU temperature
- Memory usage (used/total)
- GPU utilization percentage
- Power draw (watts)

### System Logs
- Last hour of system logs (journalctl)
- Kernel messages
- Service status changes
- Security events (sudo, ssh)

## Contributing

Contributions are welcome, though Marvin would point out that in the grand cosmic scheme, your pull request is ultimately meaningless. But please, don't let that stop you.

1. Fork the repository
2. Create a feature branch
3. Make your changes (IPMI sensors, new collectors, prompt improvements)
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - Not that licenses matter when the universe is going to end in heat death anyway.

---

*"Life! Don't talk to me about life."* - Marvin
