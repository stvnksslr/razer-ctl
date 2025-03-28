# Razer Blade Control Utlity

The goal of this project is to build a crossplatform tool for controlling razer laptops bios settings without using synapse.
One of the biggest benefits of using this tool over synapse is the ability to set your laptop bios to silent mode while on battery
which for some inexplicable reason is not available in synapse. This improved my battery life on linux from ~4 hours to a consistent 7~
on a 2023 blade 14.

# Current Support

- 2023 blades (14,15,16)
- 2024 blades (14,15,16)

# Current Features

Performance modes (including overclock)
Lid logo modes (if available): off, static, breathing
Keyboard brightness

## Usage

```sh
Usage: razer-cli <COMMAND>

Commands:
  auto       Automatically detect supported Razer device and enable device specific features
  manual     Manually specify PID of the Razer device and enable all features
  enumerate  List discovered Razer devices
  help       print the help commands

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Reverse Engineering

Read about the reverse engineering process for Razer Blade 16 in [data/README.md](data/README.md). You can follow the steps and adjust the utility for other Razer laptops.

Run `razer-cli enumerate` to get PID.
Then `razer-cli -p 0xPID info` to check if the application works for your Razer device.

Special thanks to

- [tdakhran](https://github.com/tdakhran) who created the first version of the tool
- [razer-ctl](https://github.com/tdakhran/razer-ctl) the original project that did the absurd amount of work to get this going
- [openrazer](https://github.com/openrazer) for [Reverse-Engineering-USB-Protocol](https://github.com/openrazer/openrazer/wiki/Reverse-Engineering-USB-Protocol)
- [Razer-Linux](https://github.com/Razer-Linux/razer-laptop-control-no-dkms) for USB HID protocol implementation
