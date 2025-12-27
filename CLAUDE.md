# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

razer-ctl is a cross-platform Rust utility for controlling Razer laptop BIOS settings without Synapse software. It uses USB HID protocol to communicate with the laptop's embedded controller.

## Build & Development Commands

```bash
# Build workspace (both crates)
cargo build --workspace

# Build release version (with LTO, stripped binaries)
cargo build --workspace --release

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Install CLI locally
cargo install --path blade-helper

# Run CLI directly during development
cargo run --package blade-helper -- status
cargo run --package blade-helper -- --json status
```

**Note:** On Linux, building requires `libudev-dev` for USB HID support.

## Architecture

### Workspace Structure

Two crates with clear separation of concerns:

- **librazer/** - Low-level USB HID protocol library (published to crates.io)
- **blade-helper/** - User-facing CLI application (GitHub releases only)

### librazer Core Components

| File | Purpose |
|------|---------|
| `packet.rs` | 90-byte HID packet structure with CRC calculation |
| `command.rs` | USB command implementations (get/set operations) |
| `device.rs` | USB HID device enumeration and communication |
| `descriptor.rs` | Device database - supported models, PIDs, and feature sets |
| `types.rs` | Protocol enums (PerfMode, FanMode, CpuBoost, GpuBoost, LogoMode) |
| `feature.rs` | Compile-time feature validation macros |

### blade-helper Components

| File | Purpose |
|------|---------|
| `cli.rs` | Clap-based argument parsing with subcommands |
| `device.rs` | High-level device wrapper over librazer |
| `config.rs` | Configuration storage and device PID caching |
| `settings.rs` | State structures for device settings |
| `display.rs` | Text and JSON output formatting |

### USB Protocol Details

Packets are 90 bytes structured as:
- Bytes 0-1: Status + transaction ID
- Bytes 2-7: Protocol metadata (data_size, command_class, command_id)
- Bytes 8-87: Command arguments (80 bytes)
- Byte 88: CRC (XOR of bytes 2-87)
- Byte 89: Reserved

Razer vendor ID: `0x1532`

### Adding Device Support

1. Get PID via `lsusb` (look for vendor 1532)
2. Get model number prefix from Razer support (format: RZ09-XXXXX)
3. Add `Descriptor` entry in `librazer/src/descriptor.rs` with:
   - Model prefix
   - Device name
   - USB PID
   - Supported features array

### Feature System

Features are compile-time validated using the `feature_list!` macro. Each device descriptor specifies which features it supports: BatteryCare, LidLogo, LightsAlwaysOn, KbdBacklight, Fan, Perf.

## CI/CD Pipeline

- **ci.yml**: Runs test, format check, clippy, and release build on every PR
- **release.yml**: Uses release-plz for semantic versioning
- **publish.yml**: Creates GitHub releases with cross-compiled binaries (Linux x86_64, Windows x86_64)

Release targets configured in `.goreleaser.yaml` use cargo-zigbuild for cross-compilation.

## Reverse Engineering Reference

USB packet captures and protocol documentation are in `data/`. The protocol is based on work from the openrazer project.
