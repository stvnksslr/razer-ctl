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

<!-- bv-agent-instructions-v1 -->

---

## Beads Workflow Integration

This project uses [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) for issue tracking. Issues are stored in `.beads/` and tracked in git.

### Essential Commands

```bash
# View issues (launches TUI - avoid in automated sessions)
bv

# CLI commands for agents (use these instead)
bd ready              # Show issues ready to work (no blockers)
bd list --status=open # All open issues
bd show <id>          # Full issue details with dependencies
bd create --title="..." --type=task --priority=2
bd update <id> --status=in_progress
bd close <id> --reason="Completed"
bd close <id1> <id2>  # Close multiple issues at once
bd sync               # Commit and push changes
```

### Workflow Pattern

1. **Start**: Run `bd ready` to find actionable work
2. **Claim**: Use `bd update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `bd close <id>`
5. **Sync**: Always run `bd sync` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `bd ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers, not words)
- **Types**: task, bug, feature, epic, question, docs
- **Blocking**: `bd dep add <issue> <depends-on>` to add dependencies

### Session Protocol

**Before ending any session, run this checklist:**

```bash
git status              # Check what changed
git add <files>         # Stage code changes
bd sync                 # Commit beads changes
git commit -m "..."     # Commit code
bd sync                 # Commit any new beads changes
git push                # Push to remote
```

### Best Practices

- Check `bd ready` at session start to find available work
- Update status as you work (in_progress → closed)
- Create new issues with `bd create` when you discover tasks
- Use descriptive titles and set appropriate priority/type
- Always `bd sync` before ending session

<!-- end-bv-agent-instructions -->
