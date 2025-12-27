# Razer Blade Control Utility

Cross-platform tool for controlling Razer laptop BIOS settings without Synapse.

The biggest benefit of using this tool over Synapse is the ability to set your laptop to silent mode while on battery, which for some inexplicable reason is not available in Synapse. This improved battery life on Linux from ~4 hours to a consistent 7 hours on a 2023 Blade 14.

## Supported Devices

| Model | Model Number | Features |
|-------|--------------|----------|
| Razer Blade 14" (2023) Mercury | RZ09-0482X | Performance, Fan, Keyboard, Battery Care, Lights Always On |
| Razer Blade 16" (2023) Black | RZ09-0483T | Performance, Fan, Keyboard, Logo, Battery Care, Lights Always On |

Additional models can be added - see [Adding Device Support](#adding-device-support) section.

## Features

- **Performance Modes**: Balanced, Silent, Custom (with CPU/GPU boost control)
- **Fan Control**: Auto, Manual RPM (2000-5000), Max Speed mode
- **Keyboard Brightness**: 0-255 levels
- **Lid Logo** (device-dependent): Off, Static, Breathing
- **Battery Care**: Limit charging to extend battery lifespan
- **Lights Always On**: Keep lighting active when on battery

## Installation

```bash
cargo install --path blade-helper
```

### Linux: USB Permissions

On Linux, you need udev rules to access USB devices without root:

```bash
sudo cp 99-razer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then unplug and replug your laptop (or reboot) for the rules to take effect.

## Usage

```
blade-helper [OPTIONS] <COMMAND>

Commands:
  status   Show current device status (all settings)
  get      Get a specific setting value
  set      Set a device setting
  info     Show device information
  config   Manage configuration
  help     Print help

Options:
  -v, --verbose  Enable verbose output
      --json     Output in JSON format
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

```bash
# Show all current settings
blade-helper status

# Get device info
blade-helper info

# Set performance mode to silent
blade-helper set perf silent

# Set custom performance with CPU/GPU boost
blade-helper set perf custom
blade-helper set cpu high
blade-helper set gpu medium

# Set fan to manual at 3500 RPM
blade-helper set fan manual 3500

# Set fan to auto
blade-helper set fan auto

# Enable max fan speed
blade-helper set fan max enable

# Set keyboard brightness (0-255)
blade-helper set keyboard 128

# Set logo mode (if supported)
blade-helper set logo static

# Enable battery care mode
blade-helper set battery-care enable

# Get JSON output
blade-helper --json status
```

## Adding Device Support

1. Find your device's PID using USB tools (e.g., `lsusb` on Linux)
2. Find model number prefix from [Razer support site](https://mysupport.razer.com/app/answers/detail/a_id/5481) (format: RZ09-XXXXX)
3. Add a `Descriptor` entry in `librazer/src/descriptor.rs` with supported features
4. Test with `blade-helper info` and `blade-helper status`

## Reverse Engineering

Read about the reverse engineering process for Razer Blade 16 in [data/README.md](data/README.md). You can follow the steps and adjust the utility for other Razer laptops.

## Acknowledgments

- [tdakhran](https://github.com/tdakhran) who created the first version of the tool
- [razer-ctl](https://github.com/tdakhran/razer-ctl) the original project that did the reverse engineering
- [openrazer](https://github.com/openrazer) for [Reverse-Engineering-USB-Protocol](https://github.com/openrazer/openrazer/wiki/Reverse-Engineering-USB-Protocol)
- [Razer-Linux](https://github.com/Razer-Linux/razer-laptop-control-no-dkms) for USB HID protocol implementation
