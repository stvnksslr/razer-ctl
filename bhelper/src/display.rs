use crate::device::BladeDevice;
use crate::settings::{
    DeviceState, JsonDeviceInfo, JsonDeviceState, JsonSettingValue, SettingValue,
};
use colored::*;
use librazer::types::PerfMode;

pub fn print_device_info(device: &BladeDevice) {
    println!("{}", "Device Information".bold().cyan());
    println!("  {}     {}", "Name:".dimmed(), device.name());
    println!("  {}    {}", "Model:".dimmed(), device.model());
    println!("  {}      {:#06x}", "PID:".dimmed(), device.pid());
    println!();
    println!("{}", "Supported Features:".bold().cyan());
    for feature in device.features() {
        println!("  {} {}", "•".green(), feature);
    }
}

pub fn print_device_info_json(device: &BladeDevice) {
    let info = JsonDeviceInfo {
        name: device.name().to_string(),
        model: device.model().to_string(),
        pid: format!("{:#06x}", device.pid()),
        features: device.features().iter().map(|s| s.to_string()).collect(),
    };
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}

pub fn print_status(device: &BladeDevice, state: &DeviceState) {
    println!(
        "{} {}",
        device.name().bold(),
        format!("({})", device.model()).dimmed()
    );
    println!("{}", "─".repeat(40).dimmed());

    if let Some(perf_mode) = state.perf_mode {
        let mode_color = match perf_mode {
            PerfMode::Silent => "Silent".green(),
            PerfMode::Balanced => "Balanced".yellow(),
            PerfMode::Custom => "Custom".red(),
        };
        print!("{} {}", "Performance:".dimmed(), mode_color);
        if let Some(fan_mode) = state.fan_mode {
            print!(" (Fan: {:?}", fan_mode);
            if let Some(rpm) = state.fan_rpm {
                print!(" @ {} RPM", rpm.to_string().cyan());
            }
            print!(")");
        }
        println!();

        if perf_mode == PerfMode::Custom {
            if let Some(cpu) = state.cpu_boost {
                println!("  {} {:?}", "CPU Boost:".dimmed(), cpu);
            }
            if let Some(gpu) = state.gpu_boost {
                println!("  {} {:?}", "GPU Boost:".dimmed(), gpu);
            }
        }
    }

    if let Some(max_fan) = state.max_fan_speed {
        println!("{} {:?}", "Max Fan:".dimmed(), max_fan);
    }

    if let Some(brightness) = state.keyboard_brightness {
        let bar = format_brightness_bar(brightness);
        println!("{} {} {}", "Keyboard:".dimmed(), brightness, bar);
    }

    if let Some(logo) = state.logo_mode {
        println!("{} {:?}", "Logo:".dimmed(), logo);
    }

    if let Some(care) = state.battery_care {
        let status = format!("{:?}", care);
        let colored_status = if status == "Enable" {
            status.green()
        } else {
            status.normal()
        };
        println!("{} {}", "Battery Care:".dimmed(), colored_status);
    }

    if let Some(lights) = state.lights_always_on {
        println!("{} {:?}", "Lights On:".dimmed(), lights);
    }
}

pub fn print_status_json(device: &BladeDevice, state: &DeviceState) {
    #[derive(serde::Serialize)]
    struct StatusOutput {
        device: JsonDeviceInfo,
        state: JsonDeviceState,
    }

    let output = StatusOutput {
        device: JsonDeviceInfo {
            name: device.name().to_string(),
            model: device.model().to_string(),
            pid: format!("{:#06x}", device.pid()),
            features: device.features().iter().map(|s| s.to_string()).collect(),
        },
        state: JsonDeviceState::from(state),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

pub fn print_setting(name: &str, value: &SettingValue) {
    println!("{}: {}", name.cyan(), value);
}

pub fn print_setting_json(name: &str, value: &SettingValue) {
    let output = JsonSettingValue {
        setting: name.to_string(),
        value: value.to_string(),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

pub fn print_setting_changed(name: &str, value: &SettingValue) {
    println!(
        "{} {} set to {}",
        "✓".green(),
        name.cyan(),
        value.to_string().bold()
    );
}

pub fn print_setting_changed_json(name: &str, value: &SettingValue) {
    #[derive(serde::Serialize)]
    struct ChangeOutput {
        success: bool,
        setting: String,
        value: String,
    }

    let output = ChangeOutput {
        success: true,
        setting: name.to_string(),
        value: value.to_string(),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn format_brightness_bar(brightness: u8) -> String {
    let filled = (brightness as usize * 10) / 255;
    let empty = 10 - filled;
    format!(
        "[{}{}]",
        "█".repeat(filled).cyan(),
        "░".repeat(empty).dimmed()
    )
}
