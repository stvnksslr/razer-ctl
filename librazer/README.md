# librazer

A Rust library for controlling Razer laptop BIOS settings via USB HID protocol.

[![Crates.io](https://img.shields.io/crates/v/librazer.svg)](https://crates.io/crates/librazer)
[![Documentation](https://docs.rs/librazer/badge.svg)](https://docs.rs/librazer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- Cross-platform support (Linux and Windows)
- Automatic device detection
- Performance mode control (Silent, Balanced, Custom)
- Fan control (Auto, Manual RPM, Max Speed)
- CPU/GPU boost configuration
- Keyboard backlight brightness
- Lid logo control (on supported models)
- Battery care mode
- Compile-time feature validation per device

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
librazer = "0.7"
```

### Linux Requirements

Install `libudev-dev` for USB HID support:

```bash
# Debian/Ubuntu
sudo apt install libudev-dev

# Fedora
sudo dnf install systemd-devel
```

## Usage

### Auto-detect and connect

```rust
use librazer::{device::Device, command};

fn main() -> anyhow::Result<()> {
    // Auto-detect connected Razer laptop
    let device = Device::detect()?;

    println!("Connected to: {}", device.info.name);

    // Get current performance mode
    let (perf_mode, fan_mode) = command::get_perf_mode(&device)?;
    println!("Performance: {:?}, Fan: {:?}", perf_mode, fan_mode);

    Ok(())
}
```

### Set performance mode

```rust
use librazer::{device::Device, command, types::PerfMode};

fn main() -> anyhow::Result<()> {
    let device = Device::detect()?;

    // Set to silent mode for better battery life
    command::set_perf_mode(&device, PerfMode::Silent)?;

    Ok(())
}
```

### Custom CPU/GPU boost

```rust
use librazer::{device::Device, command, types::{PerfMode, CpuBoost, GpuBoost}};

fn main() -> anyhow::Result<()> {
    let device = Device::detect()?;

    // Enable custom mode first
    command::set_perf_mode(&device, PerfMode::Custom)?;

    // Configure boost levels
    command::set_cpu_boost(&device, CpuBoost::High)?;
    command::set_gpu_boost(&device, GpuBoost::Medium)?;

    Ok(())
}
```

### Manual fan control

```rust
use librazer::{device::Device, command, types::{PerfMode, FanMode}};

fn main() -> anyhow::Result<()> {
    let device = Device::detect()?;

    // Switch to balanced mode (required for manual fan)
    command::set_perf_mode(&device, PerfMode::Balanced)?;
    command::set_fan_mode(&device, FanMode::Manual)?;

    // Set fan speed (2000-5000 RPM)
    command::set_fan_rpm(&device, 3500)?;

    Ok(())
}
```

### Other settings

```rust
use librazer::{device::Device, command, types::{BatteryCare, LogoMode, LightsAlwaysOn}};

fn main() -> anyhow::Result<()> {
    let device = Device::detect()?;

    // Enable battery care (limits charging to 80%)
    command::set_battery_care(&device, BatteryCare::Enable)?;

    // Set keyboard brightness (0-255)
    command::set_keyboard_brightness(&device, 128)?;

    // Set logo mode (if supported)
    command::set_logo_mode(&device, LogoMode::Static)?;

    // Keep lights on when sleeping
    command::set_lights_always_on(&device, LightsAlwaysOn::Enable)?;

    Ok(())
}
```

## Supported Devices

| Model | Model Number | Features |
|-------|--------------|----------|
| Razer Blade 14" (2023) Mercury | RZ09-0482X | Perf, Fan, Keyboard, Battery Care, Lights Always On |
| Razer Blade 16" (2023) Black | RZ09-0483T | Perf, Fan, Keyboard, Logo, Battery Care, Lights Always On |

## Adding Device Support

1. Find your device's USB PID: `lsusb | grep 1532`
2. Get model number from [Razer support](https://mysupport.razer.com/app/answers/detail/a_id/5481) (format: RZ09-XXXXX)
3. Add a `Descriptor` entry in `src/descriptor.rs`:

```rust
Descriptor {
    model_number_prefix: "RZ09-XXXXX",
    name: "Razer Blade XX\" (Year)",
    pid: 0xXXXX,
    features: &[
        feature::BATTERYCARE,
        feature::FAN,
        feature::KBDBACKLIGHT,
        feature::PERF,
    ],
},
```

## Protocol

Communication uses 90-byte USB HID feature reports:

| Bytes | Purpose |
|-------|---------|
| 0-1 | Status + transaction ID |
| 2-7 | Protocol metadata (data_size, command_class, command_id) |
| 8-87 | Command arguments (80 bytes) |
| 88 | CRC (XOR of bytes 2-87) |
| 89 | Reserved |

Razer vendor ID: `0x1532`

## Acknowledgments

- [tdakhran/razer-ctl](https://github.com/tdakhran/razer-ctl) - Original reverse engineering
- [openrazer](https://github.com/openrazer/openrazer) - USB protocol documentation

## License

MIT
