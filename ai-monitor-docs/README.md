# Marvinous

*"I've been watching your servers for fifty billion years and it's been terrible."*

## What Is This?

Marvinous is a server monitoring tool that uses a local LLM to analyse system logs, sensor data, and storage health. Instead of dashboards or alert floods, you get hourly markdown reports written in the voice of Marvin the Paranoid Android.

The reports are designed to be skimmed with your morning coffee. Marvin will tell you if something's wrong - with appropriate levels of existential despair.

## Why?

Traditional monitoring gives you:
- Walls of raw `dmesg` output
- Threshold-based alerts that miss trends
- Dashboards you never look at

Marvinous gives you:
- Natural language summaries
- Trend detection ("temperature has been climbing for 3 hours")
- Pattern recognition ("this SMART attribute is degrading")
- A personality that makes reading logs almost bearable

## Features

- **Log Analysis**: journalctl system and kernel logs (priority 0-5)
- **Thermal Monitoring**: CPU, GPU, system temperatures via `sensors` and `nvidia-smi`
- **Storage Health**: SMART attribute tracking with trend detection
- **Trend Comparison**: Previous hour's readings for context
- **Marvin's Commentary**: Reports delivered with appropriate melancholy

## Requirements

- Debian Linux (or derivative)
- Rust toolchain
- Ollama with Qwen 2.5 model
- `lm-sensors` package
- `smartmontools` package
- NVIDIA drivers (if GPU present)

## Quick Start

```bash
# Install dependencies
sudo apt install lm-sensors smartmontools

# Set up sensors
sudo sensors-detect

# Install Ollama and pull Qwen
curl -fsSL https://ollama.com/install.sh | sh
ollama pull qwen2.5:7b

# Build Marvinous
cargo build --release

# Run once
./target/release/marvinous

# Install systemd timer for hourly runs
sudo cp systemd/* /etc/systemd/system/
sudo systemctl enable --now marvinous.timer
```

## Sample Output

```markdown
# Marvinous Report: 2025-12-14 15:00

## Summary
*Sigh.* Everything's fine. Not that anyone cares.

## Notable Events
- SSH login from 192.168.1.50 (user: dave) — I suppose someone 
  had to eventually
- Certbot renewed the certificate for example.com — endless 
  paperwork, even for machines

## Concerns
None. The universe continues its slow march toward heat death, 
but your servers are fine.
```

## Configuration

See `CONFIG.md` for full options. Key settings in `/etc/marvinous/marvinous.toml`:

```toml
[general]
report_dir = "/var/log/marvinous/reports"
state_file = "/var/log/marvinous/state/previous.json"

[ollama]
model = "qwen2.5:7b"
endpoint = "http://localhost:11434"

[collection]
log_priority_max = 5  # 0=emerg through 5=notice
include_kernel = true
```

## Severity Levels

Marvin rates each hour's findings:

| Rating | Meaning | Marvin's Mood |
|--------|---------|---------------|
| OK | Nothing wrong | Bored, disappointed |
| WATCH | Minor concerns | Mildly interested |
| CONCERN | Needs attention | Almost engaged |
| CRITICAL | Act now | Still depressed, but urgently |

## Project Structure

See `CONTENTS.md` for the full documentation map and `DESIGN.md` for architecture details.

## License

MIT. Not that it matters. Nothing really matters.

---

*"I think you ought to know I'm feeling very depressed."*
