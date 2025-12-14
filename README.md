# Marvinous

*"Here I am, brain the size of a planet, and they ask me to monitor server metrics. Call that job satisfaction? 'Cause I don't."*

An LLM-powered server monitoring tool with the personality of Marvin the Paranoid Android from The Hitchhiker's Guide to the Galaxy.

## Overview

Marvinous watches your servers with the same enthusiasm Marvin has for... well, everything. It monitors system health, predicts failures, and reports issues with the existential dread they deserve.

**Features:**

- Real-time system metrics collection (CPU, memory, disk, network)
- LLM-powered anomaly detection and prediction
- Alerts delivered with appropriate levels of pessimism
- REST API for integration with other tools (not that they'll appreciate it)
- Systemd service for reliable operation (more reliable than the universe, anyway)

## Installation

```bash
# Clone the repository (if you must)
git clone https://github.com/lawless-m/Marvinous.git
cd Marvinous

# Build with Cargo
cargo build --release

# Install the systemd service
sudo cp marvinous.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now marvinous
```

## Configuration

Copy the example configuration:

```bash
cp config.example.toml config.toml
```

Edit `config.toml` to configure:

- Monitoring intervals
- Alert thresholds
- LLM provider settings
- Notification channels

## Usage

```bash
# Run directly
./target/release/marvinous

# Or via systemd
sudo systemctl start marvinous
sudo systemctl status marvinous

# View the logs (riveting reading, I'm sure)
journalctl -u marvinous -f
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Service health check |
| `GET /metrics` | Current system metrics |
| `GET /alerts` | Recent alerts |
| `GET /mood` | Marvin's current disposition (spoiler: it's not good) |

## Sample Alert

```
ALERT: Disk usage at 89%

"I've been watching this disk fill up for 3,247,891 seconds now.
The first ten million seconds were the worst. And the second ten
million, they were the worst too. The third ten million I didn't
enjoy at all. After that, I went into a bit of a decline.

Anyway, you might want to free up some space. Or don't.
It's not like anything matters in the end."

- Marvin
```

## Architecture

```
marvinous/
├── src/
│   ├── main.rs           # Entry point (the beginning of the end)
│   ├── collector/        # Metrics collection
│   ├── analyzer/         # LLM-powered analysis
│   ├── alerter/          # Notification dispatch
│   └── personality/      # Marvin's worldview
├── config.example.toml   # Configuration template
├── marvinous.service     # Systemd unit file
└── Cargo.toml
```

## Contributing

Contributions are welcome, though Marvin would point out that in the grand cosmic scheme, your pull request is ultimately meaningless. But please, don't let that stop you.

## License

MIT License - Not that licenses matter when the universe is going to end in heat death anyway.

---

*"Life! Don't talk to me about life."* - Marvin
