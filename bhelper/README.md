# bhelper

A command-line tool for controlling Razer laptop BIOS settings without Synapse.

[![Crates.io](https://img.shields.io/crates/v/blade-helper.svg)](https://crates.io/crates/blade-helper)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Control performance modes (Silent, Balanced, Custom)
- Manual fan speed control (2000-5000 RPM)
- CPU/GPU boost configuration
- Keyboard backlight brightness
- Lid logo control (on supported models)
- Battery care mode (limit charging to 80%)
- JSON output for scripting
- Device PID caching for faster startup

## Installation

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/stvnksslr/razer-ctl/releases).

### From crates.io

```bash
cargo install bhelper
```

### From source

```bash
git clone https://github.com/stvnksslr/razer-ctl
cd razer-ctl
cargo install --path bhelper
```

### Linux Requirements

Install udev rules for USB access without root:

```bash
# Create udev rule
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="1532", MODE="0666"' | sudo tee /etc/udev/rules.d/99-razer.rules

# Reload rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Also requires `libudev-dev` if building from source:

```bash
# Debian/Ubuntu
sudo apt install libudev-dev

# Fedora
sudo dnf install systemd-devel
```

## Usage

### View current status

```bash
blade-helper status
```

Output:
```
Device: Razer Blade 14" (2023) Mercury

Performance Mode:  Balanced
Fan Mode:          Auto
Fan RPM:           2800
CPU Boost:         Low
GPU Boost:         Low
Keyboard:          128
Battery Care:      Enabled
Lights Always On:  Disabled
```

### Get a specific setting

```bash
blade-helper get perf
blade-helper get fan
blade-helper get keyboard
```

### Set performance mode

```bash
# Silent mode - quiet, battery-friendly
blade-helper set perf silent

# Balanced mode - default
blade-helper set perf balanced

# Custom mode - enables manual CPU/GPU control
blade-helper set perf custom
```

### Control fan speed

```bash
# Automatic fan control
blade-helper set fan auto

# Manual RPM (2000-5000)
blade-helper set fan manual 3500

# Max fan speed mode
blade-helper set fan max on
blade-helper set fan max off
```

### CPU/GPU boost (requires custom perf mode)

```bash
# Set CPU boost level
blade-helper set cpu low
blade-helper set cpu medium
blade-helper set cpu high
blade-helper set cpu boost

# Set GPU boost level
blade-helper set gpu low
blade-helper set gpu medium
blade-helper set gpu high
```

### Keyboard backlight

```bash
# Set brightness (0-255)
blade-helper set keyboard 128

# Turn off
blade-helper set keyboard 0
```

### Logo control (supported models only)

```bash
blade-helper set logo off
blade-helper set logo static
blade-helper set logo breathing
```

### Battery care

```bash
# Enable (limits charging to 80%)
blade-helper set battery-care on

# Disable
blade-helper set battery-care off
```

### Lights always on

```bash
# Keep keyboard lit during sleep
blade-helper set lights-always-on on
blade-helper set lights-always-on off
```

## JSON Output

Add `--json` for machine-readable output:

```bash
blade-helper --json status
blade-helper --json get fan
```

## Configuration

Configuration is stored at:
- Linux: `~/.config/blade-helper/config.toml`
- Windows: `%APPDATA%\blade-helper\config.toml`

```bash
# Show configuration
blade-helper config show

# Show config file path
blade-helper config path

# Clear device cache (forces re-detection)
blade-helper config clear-cache
```

## Supported Devices

| Model | Model Number |
|-------|--------------|
| Razer Blade 14" (2023) Mercury | RZ09-0482X |
| Razer Blade 16" (2023) Black | RZ09-0483T |

See [librazer](https://crates.io/crates/librazer) for adding device support.

## Troubleshooting

### Permission denied (Linux)

Ensure udev rules are installed (see Installation section).

### Device not found

1. Run `lsusb | grep 1532` to verify device is connected
2. Try `blade-helper config clear-cache` to force re-detection
3. Check if your device is supported

### Verbose output

```bash
blade-helper -v status
```

## License

MIT
