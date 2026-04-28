# X1 ClockDrift

Live measurement of `Clock::unix_timestamp` drift on the X1 blockchain.

**Live dashboard**: https://piowin-clo.github.io/x1-clockdrift/

[Polski README](README.pl.md) · [Methodology](docs/methodology.md)

## What this measures

X1 (an SVM-fork of Solana, Tachyon validator client) produces a chain-wide
`unix_timestamp` per slot. That value is the stake-weighted median of
timestamps reported by all active validators in their vote instructions.
This project measures, per validator, how far each validator's reported
timestamp drifts from real UTC observed locally.

For every slot frozen on the host validator, we record the local clock
reading. From later blocks we collect every vote instruction, each of
which carries the voting validator's reported timestamp for some prior
slot. By joining these, we derive `drift = chain_timestamp − local_clock`
for every (validator, slot) pair we can match.

## Architecture

- The daemon runs on a single X1 validator (the host).
- It tails `validator.log` to obtain microsecond-precision local clock
  readings of `bank frozen` events.
- It queries the local RPC (`http://localhost:8899`) at no more than 5
  requests/second for vote instructions in newly-frozen blocks.
- A SQLite database accumulates raw observations and refreshed stake
  snapshots.
- Every 5 minutes the daemon recomputes per-validator and per-network
  aggregates, exports JSON to a clone of this repository, and pushes
  to the `data` branch.
- The dashboard is the contents of the `data` branch served by GitHub
  Pages.

## Repository layout

```
.
├── daemon/                Rust daemon (binary x1cd)
├── frontend/              Vanilla HTML + JS dashboard
├── install/               Install scripts and systemd unit
├── docs/                  Methodology
└── .github/workflows/     CI + Pages deploy
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

## Installing on a validator

See [install/install.sh](install/install.sh). Summary:

```bash
sudo -u x1pio bash install/install.sh
sudo systemctl start x1cd
journalctl -u x1cd -f
```

The installer generates an SSH deploy key, asks the operator to register
it on GitHub with write access, clones the `data` branch, writes
`config.toml`, and installs the systemd unit.

## Operational guarantees

- `CPUQuota=20%`, `MemoryMax=512M`, `Nice=19`, `IOSchedulingClass=idle`:
  the validator always has priority.
- `Type=notify` + `WatchdogSec=120`: systemd kills the daemon if it stops
  pinging.
- `touch /home/x1pio/strontium-meter/STOP` causes a clean exit within 5 s.

## Status

[![CI](https://github.com/PioWin-clo/x1-clockdrift/actions/workflows/ci.yml/badge.svg)](https://github.com/PioWin-clo/x1-clockdrift/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
