mod cli;
mod config;
mod device;
mod display;
mod error;
mod settings;

use clap::Parser;
use colored::*;
use librazer::types::FanMode;
use log::debug;

use cli::{Cli, Commands, ConfigCommand, FanCommand, SetCommand, SettingName};
use config::ConfigManager;
use device::BladeDevice;
use error::Result;
use settings::{Setting, SettingValue};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        if std::env::var("NO_COLOR").is_ok() {
            eprintln!("Error: {}", e);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    // Initialize logging based on verbosity
    let log_level = if cli.verbose { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    debug!("Parsed CLI arguments");

    let json = cli.json;

    match cli.command {
        Commands::Status => cmd_status(json)?,
        Commands::Get { setting } => cmd_get(setting, json)?,
        Commands::Set { setting } => cmd_set(setting, json)?,
        Commands::Info => cmd_info(json)?,
        Commands::Config { action } => cmd_config(action, json)?,
    }

    Ok(())
}

fn cmd_status(json: bool) -> Result<()> {
    let device = BladeDevice::detect_with_cache()?;
    let state = device.read_state()?;
    if json {
        display::print_status_json(&device, &state);
    } else {
        display::print_status(&device, &state);
    }
    Ok(())
}

fn cmd_get(setting: SettingName, json: bool) -> Result<()> {
    let device = BladeDevice::detect_with_cache()?;

    let (name, setting_type) = match setting {
        SettingName::Perf => ("Performance Mode", Setting::PerfMode),
        SettingName::Cpu => ("CPU Boost", Setting::CpuBoost),
        SettingName::Gpu => ("GPU Boost", Setting::GpuBoost),
        SettingName::Fan => ("Fan", Setting::FanMode),
        SettingName::MaxFan => ("Max Fan Speed", Setting::MaxFanSpeed),
        SettingName::Keyboard => ("Keyboard Brightness", Setting::KeyboardBrightness),
        SettingName::Logo => ("Logo Mode", Setting::LogoMode),
        SettingName::BatteryCare => ("Battery Care", Setting::BatteryCare),
        SettingName::LightsAlwaysOn => ("Lights Always On", Setting::LightsAlwaysOn),
    };

    let value = device.get_setting(setting_type)?;
    if json {
        display::print_setting_json(name, &value);
    } else {
        display::print_setting(name, &value);
    }
    Ok(())
}

fn cmd_set(setting: SetCommand, json: bool) -> Result<()> {
    let device = BladeDevice::detect_with_cache()?;

    let (name, value) = match setting {
        SetCommand::Perf { mode } => (
            "Performance Mode",
            SettingValue::PerfMode {
                mode,
                fan_mode: FanMode::Auto,
            },
        ),
        SetCommand::Cpu { boost } => ("CPU Boost", SettingValue::CpuBoost(boost)),
        SetCommand::Gpu { boost } => ("GPU Boost", SettingValue::GpuBoost(boost)),
        SetCommand::Fan { action } => {
            let value = match action {
                FanCommand::Auto => SettingValue::Fan {
                    mode: FanMode::Auto,
                    rpm: None,
                },
                FanCommand::Manual { rpm } => SettingValue::Fan {
                    mode: FanMode::Manual,
                    rpm: Some(rpm),
                },
                FanCommand::Max { mode } => SettingValue::MaxFanSpeed(mode),
            };

            if matches!(value, SettingValue::MaxFanSpeed(_)) {
                ("Max Fan Speed", value)
            } else {
                ("Fan", value)
            }
        }
        SetCommand::Keyboard { brightness } => (
            "Keyboard Brightness",
            SettingValue::KeyboardBrightness(brightness),
        ),
        SetCommand::Logo { mode } => ("Logo Mode", SettingValue::LogoMode(mode)),
        SetCommand::BatteryCare { mode } => ("Battery Care", SettingValue::BatteryCare(mode)),
        SetCommand::LightsAlwaysOn { mode } => {
            ("Lights Always On", SettingValue::LightsAlwaysOn(mode))
        }
    };

    device.apply_setting(value.clone())?;
    if json {
        display::print_setting_changed_json(name, &value);
    } else {
        display::print_setting_changed(name, &value);
    }
    Ok(())
}

fn cmd_info(json: bool) -> Result<()> {
    let device = BladeDevice::detect_with_cache()?;
    if json {
        display::print_device_info_json(&device);
    } else {
        display::print_device_info(&device);
    }
    Ok(())
}

fn cmd_config(action: ConfigCommand, json: bool) -> Result<()> {
    match action {
        ConfigCommand::Show => {
            let config_mgr = ConfigManager::load()?;
            let config = config_mgr.config();

            if json {
                #[derive(serde::Serialize)]
                struct ConfigOutput {
                    path: String,
                    device_cache: DeviceCacheOutput,
                    settings: SettingsOutput,
                }
                #[derive(serde::Serialize)]
                struct DeviceCacheOutput {
                    pid: Option<String>,
                    model: Option<String>,
                    model_prefix: Option<String>,
                }
                #[derive(serde::Serialize)]
                struct SettingsOutput {
                    default_profile: Option<String>,
                }

                let output = ConfigOutput {
                    path: config_mgr.path().display().to_string(),
                    device_cache: DeviceCacheOutput {
                        pid: config.device.cached_pid.map(|p| format!("{:#06x}", p)),
                        model: config.device.model.clone(),
                        model_prefix: config.device.model_prefix.clone(),
                    },
                    settings: SettingsOutput {
                        default_profile: config.settings.default_profile.clone(),
                    },
                };
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!("{}", "Configuration:".bold().cyan());
                println!(
                    "  {} {}",
                    "Config file:".dimmed(),
                    config_mgr.path().display()
                );
                println!();

                println!("{}", "Device Cache:".bold().cyan());
                if let Some(pid) = config.device.cached_pid {
                    println!("  {} {:#06x}", "PID:".dimmed(), pid);
                    if let Some(model) = &config.device.model {
                        println!("  {} {}", "Model:".dimmed(), model);
                    }
                    if let Some(prefix) = &config.device.model_prefix {
                        println!("  {} {}", "Model Prefix:".dimmed(), prefix);
                    }
                } else {
                    println!("  {}", "(no cached device)".dimmed());
                }
                println!();

                println!("{}", "Settings:".bold().cyan());
                if let Some(profile) = &config.settings.default_profile {
                    println!("  {} {}", "Default Profile:".dimmed(), profile);
                } else {
                    println!("  {} {}", "Default Profile:".dimmed(), "(none)".dimmed());
                }
            }
        }
        ConfigCommand::SetDefault { profile } => {
            let mut config_mgr = ConfigManager::load()?;
            config_mgr.config_mut().settings.default_profile = Some(profile.clone());
            config_mgr.save()?;
            if json {
                println!(r#"{{"success": true, "default_profile": "{}"}}"#, profile);
            } else {
                println!(
                    "{} Default profile set to '{}'",
                    "✓".green(),
                    profile.cyan()
                );
            }
        }
        ConfigCommand::ClearCache => {
            let mut config_mgr = ConfigManager::load()?;
            config_mgr.clear_cache()?;
            if json {
                println!(r#"{{"success": true, "message": "Device cache cleared"}}"#);
            } else {
                println!("{} Device cache cleared", "✓".green());
            }
        }
        ConfigCommand::Path => {
            let path = ConfigManager::config_path()?;
            if json {
                println!(r#"{{"path": "{}"}}"#, path.display());
            } else {
                println!("{}", path.display());
            }
        }
    }

    Ok(())
}
