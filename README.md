# Razer Blade control utility

* own a Razer Blade 16 2023 or Razor Blade 14 2023
* use Windows 11 or Linux
* want better battery life ex. synapse doesnt allow silent mode on battery depsite it making a huge difference

I have great news. I've reverse-engineered the Razer protocol and have crafted an alternative solution. A drop-in predictable and compact Razer Synapse alternative.

## What can it control?

* Performance modes (including overclock)
* Lid logo modes: off, static, breathing
* Keyboard brightness (works on Windows with Fn keys anyway)

## linux support

work in progress, tray is not supported but cli currently works however it requires sudo for the moment to access the various HID
devices.

## Usage

```sh
Usage: razer-cli <COMMAND>

Commands:
  auto       Automatically detect supported Razer device and enable device specific features
  manual     Manually specify PID of the Razer device and enable all features (many might not work)
  enumerate  List discovered Razer devices
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Reverse Engineering

Read about the reverse engineering process for Razer Blade 16 in [data/README.md](data/README.md). You can follow the steps and adjust the utility for other Razer laptops.

Run `razer-cli enumerate` to get PID.
Then `razer-cli -p 0xPID info` to check if the application works for your Razer device.

Special thanks to

* [razer-ctl](https://github.com/tdakhran/razer-ctl) the original project that did the absurd amount of work to get this going
* [openrazer](https://github.com/openrazer) for [Reverse-Engineering-USB-Protocol](https://github.com/openrazer/openrazer/wiki/Reverse-Engineering-USB-Protocol)
* [Razer-Linux](https://github.com/Razer-Linux/razer-laptop-control-no-dkms) for USB HID protocol implementation
