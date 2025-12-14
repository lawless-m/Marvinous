# Marvinous - Documentation Contents

*"Here I am, brain the size of a planet, and they ask me to watch log files."*

## Start Here

**Marvinous** is a Rust-based server monitoring tool that uses a local LLM (Qwen via Ollama) to analyse system logs and sensor data. Reports are delivered in the voice of Marvin the Paranoid Android - deeply pessimistic, perpetually bored, but grudgingly competent.

## Document Overview

| File | Purpose |
|------|---------|
| `README.md` | Project overview, goals, quick start |
| `DESIGN.md` | Architecture, data flow, component design |
| `SPEC.md` | Technical specification, data formats, interfaces |
| `PROMPT.md` | LLM prompt template and output format |
| `CONFIG.md` | Configuration options and defaults |
| `EXAMPLE-OUTPUT.md` | Sample reports at different severity levels |

## Reading Order

1. **README.md** - Understand what we're building
2. **DESIGN.md** - Understand how it fits together
3. **SPEC.md** - Implementation details
4. **PROMPT.md** - The LLM interaction (core of the system)
5. **CONFIG.md** - Reference as needed

## Key Decisions Already Made

- **Language**: Rust
- **LLM**: Qwen 2.5 via Ollama (local)
- **Target OS**: Debian Linux
- **Output**: Markdown reports, human-scannable
- **Scheduling**: Systemd timer (hourly)
- **No Python** - Rust throughout

## Project Structure (Target)

```
marvinous/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── collector/
│   │   ├── mod.rs
│   │   ├── journalctl.rs
│   │   ├── sensors.rs
│   │   ├── nvidia.rs
│   │   └── smart.rs
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── ollama.rs
│   │   └── prompt.rs
│   ├── output/
│   │   ├── mod.rs
│   │   ├── report.rs
│   │   └── state.rs
│   └── config.rs
├── config/
│   └── marvinous.toml
├── systemd/
│   ├── marvinous.service
│   └── marvinous.timer
└── prompts/
    └── system-prompt.txt
```

## Runtime File Locations

```
/var/log/marvinous/
├── reports/
│   └── YYYY-MM-DD-HH.md
└── state/
    └── previous.json
```
