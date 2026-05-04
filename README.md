# X1-ClockDrift

Public time-integrity monitor for the X1 blockchain.

**Live dashboard:** <https://piowin-clo.github.io/X1-ClockDrift/>

**Methodology:** <https://piowin-clo.github.io/X1-ClockDrift/docs/methodology.html>

## What it does

X1-ClockDrift measures and visualizes time-related signals on X1 mainnet:

- **Layer 1 — Vote pipeline latency** (~400-850 ms baseline). The
  inherent time it takes a validator's vote to appear on-chain on
  Tachyon (the Solana fork powering X1). Sum of signing + gossip +
  block inclusion. Identical across well-synchronized validators.
- **Layer 2 — Clock drift** (rare). Validators whose system clocks are
  misconfigured by 5+ seconds — genuine NTP/chrony issues operators
  can fix.
- **Foundation operational changes.** The 12-node X1 Labs cluster is
  monitored as a baseline; sudden shifts in the foundation pipeline
  trend indicate config changes, deployments, or load events.

The dashboard explicitly distinguishes Layer 1 (pipeline latency,
normal) from Layer 2 (clock drift, problematic) — a distinction that
matters for correctly interpreting time-related signals on the chain,
and one that earlier versions of this dashboard conflated.

See the [methodology page](https://piowin-clo.github.io/X1-ClockDrift/docs/methodology.html)
for the full framework, atomic time sources, drift formula, and
limitations.

## Architecture

- **Daemon** (`x1cd`, in `daemon/`) — Rust service running on a
  Sentinel host with chrony synchronized to multiple stratum-1 NTP
  sources (PTB Germany, GUM Poland, CESNET Czechia, Netnod Sweden).
  Tails `validator.log` for microsecond-precision local clock
  readings of `bank frozen` events for every slot, and queries the
  public X1 RPC (`https://rpc.mainnet.x1.xyz`) for vote instructions
  via `getBlock` with `encoding=jsonParsed`.
- **Storage** — SQLite, ~1 GB after several months of operation.
- **Exporter** — recomputes per-validator and per-network aggregates
  every 5 minutes, writes JSON, and pushes to the `data` branch via
  an SSH deploy key.
- **Frontend** — vanilla HTML/CSS/JS (`frontend/`) served via GitHub
  Pages from the `data` branch. No backend API: the dashboard reads
  the JSON files directly.

### Exported JSON files

| File | Contents |
|------|---|
| `summary.json` | Network-level aggregates, drift bands, foundation count |
| `validators.json` | Per-validator drift summary |
| `history.json` | 7 days × 5-minute buckets of network drift |
| `meta.json` | Daemon version + observation totals |
| `best_validators.json` | Top 10 lowest pipeline latency |
| `pipeline_anomalies.json` | **v1.0.0** Tier 1: 500 ms ≤ \|lag\| < 5 s |
| `clock_drift.json` | **v1.0.0** Tier 2: \|drift\| ≥ 5 s — Layer 2 |
| `worst_validators.json` | Legacy combined ranking — DEPRECATED in v1.0.0, removal scheduled v1.1.0 |
| `foundation.json` | 12-node X1 Labs cluster snapshot |
| `foundation_drift_trend.json` | 14 days × 1-hour buckets of foundation cluster pipeline trend |
| `chrony.json` | Sentinel chrony tracking + NTP source state |

## Repository layout

```
.
├── daemon/                Rust daemon (binary x1cd)
├── frontend/              Vanilla HTML + JS dashboard
│   └── docs/
│       └── methodology.html  Layer 1/2 framework & measurement docs
├── install/               Install scripts and systemd unit
└── .github/workflows/     CI + release workflows
```

## Building from source

```bash
cargo build --release
```

The binary lands at `target/release/x1cd`. Static binaries for Linux
x86_64 are produced by the GitHub Actions release workflow.

## Running locally (development)

```bash
cargo run --bin x1cd -- --config ./config.toml run
```

The HTTP API and dashboard are served on `127.0.0.1:8088` (configurable).
The daemon will refuse to start if the configured kill-switch file
exists.

## Self-hosting (validator operators)

Operators who want their own instance can install via the bundled
script:

```bash
sudo -u x1pio bash install/install.sh
sudo systemctl start x1cd
journalctl -u x1cd -f
```

The installer downloads the latest release tarball, verifies its
SHA256, drops the binary into `/home/x1pio/strontium-meter/bin/`,
generates an SSH deploy key (operator registers it on GitHub with
write access), clones the `data` branch, writes `config.toml`, and
installs the systemd unit.

Requirements:
- Linux x86_64 with glibc 2.35+ (Ubuntu 22.04 baseline)
- chrony installed and synchronized to stratum-1 sources
- Solana RPC access to X1 mainnet
- Optional: SSH deploy key for pushing to your own GitHub Pages branch

Releases: <https://github.com/PioWin-clo/X1-ClockDrift/releases/latest>

## Operational guarantees

- `CPUQuota=20%`, `MemoryMax=512M`, `Nice=19`, `IOSchedulingClass=idle`:
  the validator always has priority.
- `Type=notify` + `WatchdogSec=120`: systemd kills the daemon if it
  stops pinging.
- `touch /home/x1pio/strontium-meter/STOP` causes a clean exit within 5 s.

## License

Apache-2.0 — see [LICENSE](LICENSE).

## Acknowledgments

- The X1 Labs team, particularly Theo, for clarifying the Layer 1 vs
  Layer 2 framework that informs this dashboard's interpretation
  (personal communication, X1 Labs Telegram).
- Solana Labs for the underlying consensus mechanics.
- The chrony project for the precision time synchronization that
  makes reliable measurement possible.

## Contact

GitHub issues for bugs and feature requests. Architecture questions
welcome.

## Status

[![CI](https://github.com/PioWin-clo/X1-ClockDrift/actions/workflows/ci.yml/badge.svg)](https://github.com/PioWin-clo/X1-ClockDrift/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
