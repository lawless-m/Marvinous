# Marvinous Build and Deploy Guide

> A step-by-step guide for building and deploying Marvinous on a Linux server.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [System Dependencies](#system-dependencies)
3. [Install Ollama and LLM Model](#install-ollama-and-llm-model)
4. [Build Marvinous](#build-marvinous)
5. [Create Directory Structure](#create-directory-structure)
6. [Install Binary and Configuration](#install-binary-and-configuration)
7. [Configure Marvinous](#configure-marvinous)
8. [Set Up Systemd Services](#set-up-systemd-services)
9. [Verify Installation](#verify-installation)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

| Component | Minimum Version | Purpose |
|-----------|-----------------|---------|
| Linux | Ubuntu 20.04+ / Debian 11+ / RHEL 8+ | Operating system |
| Rust | 1.70+ | Build toolchain |
| systemd | 245+ | Service management |
| Git | 2.25+ | Source retrieval |

### Hardware Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 8 GB | 16 GB |
| VRAM (GPU) | 6 GB | 8+ GB |
| Disk | 20 GB free | 50 GB free |
| CPU | 4 cores | 8+ cores |

> **Note:** Marvinous can run without a GPU using CPU-only Ollama inference, but response times will be significantly slower.

---

## System Dependencies

### Debian/Ubuntu

```bash
# Update package lists
sudo apt update

# Install build essentials and monitoring tools
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    lm-sensors \
    smartmontools \
    curl \
    git

# Initialize hardware sensors
sudo sensors-detect --auto
```

### RHEL/CentOS/Fedora

```bash
# Install development tools and monitoring utilities
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y \
    openssl-devel \
    lm_sensors \
    smartmontools \
    curl \
    git

# Initialize hardware sensors
sudo sensors-detect --auto
```

### Arch Linux

```bash
sudo pacman -S --needed \
    base-devel \
    openssl \
    lm_sensors \
    smartmontools \
    curl \
    git

sudo sensors-detect --auto
```

### Install Rust (if not present)

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

---

## Install Ollama and LLM Model

### Install Ollama

```bash
# Download and run Ollama installer
curl -fsSL https://ollama.com/install.sh | sh

# Verify Ollama is running
systemctl status ollama

# If not running, start it
sudo systemctl enable --now ollama
```

### Pull the Recommended Model

```bash
# Pull Qwen 2.5 7B (recommended - requires ~6GB VRAM)
ollama pull qwen2.5:7b

# Verify model is available
ollama list
```

#### Alternative Models

| Model | VRAM Required | Command |
|-------|---------------|---------|
| qwen2.5:7b | 6 GB | `ollama pull qwen2.5:7b` |
| qwen2.5:14b | 12 GB | `ollama pull qwen2.5:14b` |
| llama3.1:8b | 6 GB | `ollama pull llama3.1:8b` |
| mistral:7b | 6 GB | `ollama pull mistral:7b` |

### Verify Ollama is Working

```bash
# Test that Ollama responds
curl -s http://localhost:11434/api/tags | head -c 200

# Test model inference
ollama run qwen2.5:7b "Say hello in one sentence"
```

---

## Build Marvinous

### Clone the Repository

```bash
# Clone to a build directory
cd /tmp
git clone https://github.com/lawless-m/Marvinous.git
cd Marvinous
```

### Build Release Binary

```bash
# Build optimized release binary
cargo build --release

# Verify the binary was created
ls -la target/release/marvinous
```

The release build includes:
- Link-time optimization (LTO)
- Symbol stripping
- Optimized binary size (~3-5 MB)

---

## Create Directory Structure

```bash
# Create configuration directory
sudo mkdir -p /etc/marvinous

# Create log and state directories
sudo mkdir -p /var/log/marvinous/reports
sudo mkdir -p /var/log/marvinous/state

# Set appropriate permissions
sudo chmod 755 /etc/marvinous
sudo chmod 755 /var/log/marvinous
sudo chmod 755 /var/log/marvinous/reports
sudo chmod 755 /var/log/marvinous/state
```

---

## Install Binary and Configuration

### Install the Binary

```bash
# Copy binary to system path
sudo cp target/release/marvinous /usr/local/bin/

# Make it executable
sudo chmod 755 /usr/local/bin/marvinous

# Verify installation
/usr/local/bin/marvinous --help
```

### Install Configuration Files

```bash
# Copy example configuration
sudo cp config/marvinous.toml /etc/marvinous/marvinous.toml

# Copy system prompt
sudo cp prompts/system-prompt.txt /etc/marvinous/system-prompt.txt

# Set file permissions
sudo chmod 644 /etc/marvinous/marvinous.toml
sudo chmod 644 /etc/marvinous/system-prompt.txt
```

---

## Configure Marvinous

### Edit Configuration File

```bash
sudo nano /etc/marvinous/marvinous.toml
```

### Configuration Reference

```toml
[general]
# Directory for hourly reports (YYYY-MM-DD-HH.md)
report_dir = "/var/log/marvinous/reports"

# State file for trend tracking between runs
state_file = "/var/log/marvinous/state/previous.json"

# Path to LLM system prompt
prompt_file = "/etc/marvinous/system-prompt.txt"

# Logging verbosity: error, warn, info, debug, trace
log_level = "info"

[ollama]
# Ollama API endpoint
endpoint = "http://localhost:11434"

# LLM model to use
model = "qwen2.5:7b"

# Timeout for LLM responses (seconds)
timeout_secs = 120

[collection]
# How far back to collect logs
log_since = "1 hour ago"

# Maximum log priority (0=emergency through 7=debug)
# 5 = notice and higher
log_priority_max = 5

# Include kernel messages
include_kernel = true

# Maximum log entries to send to LLM
max_log_entries = 500

[storage]
# Storage devices to monitor
# Empty array = auto-detect all drives
devices = []

[sensors]
# Enable hardware sensor collection
enabled = true

# Use JSON output format (recommended)
json_format = true

[gpu]
# Enable NVIDIA GPU monitoring
enabled = true

# Don't fail if no GPU present
optional = true

[notifications]
# Future feature - not yet implemented
enabled = false
```

### Environment Variable Overrides

These environment variables override configuration file settings:

| Variable | Overrides |
|----------|-----------|
| `MARVINOUS_REPORT_DIR` | `general.report_dir` |
| `MARVINOUS_OLLAMA_ENDPOINT` | `ollama.endpoint` |
| `MARVINOUS_OLLAMA_MODEL` | `ollama.model` |
| `MARVINOUS_LOG_LEVEL` | `general.log_level` |

---

## Set Up Systemd Services

### Install Systemd Units

```bash
# Copy service and timer files
sudo cp systemd/marvinous.service /etc/systemd/system/
sudo cp systemd/marvinous.timer /etc/systemd/system/

# Reload systemd daemon
sudo systemctl daemon-reload
```

### Review Service Configuration

**marvinous.service** - Runs once per trigger:
```ini
[Unit]
Description=Marvinous Server Monitor
After=network.target ollama.service
Wants=ollama.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/marvinous
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

**marvinous.timer** - Triggers hourly:
```ini
[Unit]
Description=Run Marvinous hourly

[Timer]
OnCalendar=hourly
Persistent=true

[Install]
WantedBy=timers.target
```

### Enable and Start Timer

```bash
# Enable timer to start on boot
sudo systemctl enable marvinous.timer

# Start timer now
sudo systemctl start marvinous.timer

# Verify timer is active
sudo systemctl status marvinous.timer
```

---

## Verify Installation

### Run Manual Test

```bash
# Test with dry-run (collect data, don't call LLM)
sudo /usr/local/bin/marvinous --dry-run

# Test showing prompt that would be sent
sudo /usr/local/bin/marvinous --show-prompt

# Run full execution
sudo /usr/local/bin/marvinous
```

### Check Generated Report

```bash
# List reports
ls -la /var/log/marvinous/reports/

# View most recent report
cat /var/log/marvinous/reports/$(ls -t /var/log/marvinous/reports/ | head -1)
```

### Check Systemd Status

```bash
# Check timer status and next run time
sudo systemctl list-timers marvinous.timer

# Check service logs
sudo journalctl -u marvinous.service -n 50

# Check timer logs
sudo journalctl -u marvinous.timer -n 20
```

### Verify All Components

```bash
# Check sensors work
sensors

# Check SMART data (replace sdX with actual device)
sudo smartctl -A /dev/sda

# Check GPU (if NVIDIA present)
nvidia-smi --query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total --format=csv

# Check Ollama health
curl -s http://localhost:11434/api/tags
```

---

## Troubleshooting

### Exit Codes

| Code | Meaning | Resolution |
|------|---------|------------|
| 0 | Success | No action needed |
| 1 | Configuration error | Check `/etc/marvinous/marvinous.toml` syntax |
| 2 | Collection error | Check sensor/smartctl permissions |
| 3 | Ollama unreachable | Verify `systemctl status ollama` |
| 4 | Write error | Check disk space and directory permissions |

### Common Issues

#### Ollama Not Responding

```bash
# Check if Ollama is running
sudo systemctl status ollama

# Restart Ollama
sudo systemctl restart ollama

# Check Ollama logs
sudo journalctl -u ollama -n 50
```

#### Permission Denied for SMART Data

```bash
# Marvinous needs root to read SMART data
# Verify service runs as root (check marvinous.service)
sudo /usr/local/bin/marvinous --dry-run
```

#### Sensors Not Detected

```bash
# Re-run sensor detection
sudo sensors-detect --auto

# Load kernel modules
sudo systemctl restart lm-sensors

# Verify sensors work
sensors
```

#### No GPU Detected

If you don't have an NVIDIA GPU, ensure the configuration has:

```toml
[gpu]
enabled = true
optional = true  # This prevents failure when no GPU
```

#### LLM Timeout

Increase the timeout in configuration:

```toml
[ollama]
timeout_secs = 180  # Increase from 120 to 180
```

Or use a smaller/faster model:

```toml
[ollama]
model = "qwen2.5:7b"  # Instead of 14b or 32b
```

#### Reports Not Being Generated

```bash
# Check directory permissions
ls -la /var/log/marvinous/

# Check disk space
df -h /var/log

# Run manually with verbose logging
MARVINOUS_LOG_LEVEL=debug sudo /usr/local/bin/marvinous
```

### Log Locations

| Log | Location | Command |
|-----|----------|---------|
| Service output | systemd journal | `sudo journalctl -u marvinous.service` |
| Timer events | systemd journal | `sudo journalctl -u marvinous.timer` |
| Reports | `/var/log/marvinous/reports/` | `ls -lt /var/log/marvinous/reports/` |
| State file | `/var/log/marvinous/state/` | `cat /var/log/marvinous/state/previous.json` |

---

## Quick Reference

### File Locations Summary

| Component | Path |
|-----------|------|
| Binary | `/usr/local/bin/marvinous` |
| Configuration | `/etc/marvinous/marvinous.toml` |
| System Prompt | `/etc/marvinous/system-prompt.txt` |
| Reports | `/var/log/marvinous/reports/` |
| State | `/var/log/marvinous/state/previous.json` |
| Service Unit | `/etc/systemd/system/marvinous.service` |
| Timer Unit | `/etc/systemd/system/marvinous.timer` |

### Useful Commands

```bash
# Run Marvinous manually
sudo /usr/local/bin/marvinous

# Run with debug output
MARVINOUS_LOG_LEVEL=debug sudo /usr/local/bin/marvinous

# Test without LLM call
sudo /usr/local/bin/marvinous --dry-run

# Check timer status
sudo systemctl list-timers marvinous.timer

# View latest report
cat /var/log/marvinous/reports/$(ls -t /var/log/marvinous/reports/ | head -1)

# Restart Ollama
sudo systemctl restart ollama

# View service logs
sudo journalctl -u marvinous.service -f
```

---

## Uninstallation

To completely remove Marvinous:

```bash
# Stop and disable services
sudo systemctl stop marvinous.timer
sudo systemctl disable marvinous.timer
sudo systemctl stop marvinous.service

# Remove systemd units
sudo rm /etc/systemd/system/marvinous.service
sudo rm /etc/systemd/system/marvinous.timer
sudo systemctl daemon-reload

# Remove binary
sudo rm /usr/local/bin/marvinous

# Remove configuration
sudo rm -rf /etc/marvinous

# Remove logs (optional - preserves historical reports)
sudo rm -rf /var/log/marvinous
```

---

*"I've seen things you people wouldn't believe. System logs on fire off the shoulder of Orion. I watched SMART values glitter in the dark near the Tannhauser Gate. All those moments will be captured in markdown... like tears in rain."* — Marvinous
