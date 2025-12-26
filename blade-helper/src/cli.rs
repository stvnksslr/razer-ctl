use clap::{Parser, Subcommand, ValueEnum};
use librazer::types::{
    BatteryCare, CpuBoost, GpuBoost, LightsAlwaysOn, LogoMode, MaxFanSpeedMode, PerfMode,
};

#[derive(Parser)]
#[command(name = "blade_helper")]
#[command(
    author,
    version,
    about = "friendly CLI for controlling Razer laptop settings"
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show current device status (all settings)
    Status,

    /// Get a specific setting value
    Get {
        /// The setting to retrieve
        #[arg(value_enum)]
        setting: SettingName,
    },

    /// Set a device setting
    Set {
        #[command(subcommand)]
        setting: SetCommand,
    },

    /// Show device information
    Info,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
}

#[derive(Subcommand)]
pub enum SetCommand {
    /// Set performance mode (balanced, silent, custom)
    Perf {
        #[arg(value_enum)]
        mode: PerfMode,
    },

    /// Set CPU boost level (requires custom perf mode)
    Cpu {
        #[arg(value_enum)]
        boost: CpuBoost,
    },

    /// Set GPU boost level (requires custom perf mode)
    Gpu {
        #[arg(value_enum)]
        boost: GpuBoost,
    },

    /// Set fan speed (RPM) or mode
    Fan {
        #[command(subcommand)]
        action: FanCommand,
    },

    /// Set keyboard backlight brightness (0-255)
    Keyboard {
        /// Brightness level (0-255)
        #[arg(value_parser = clap::value_parser!(u8))]
        brightness: u8,
    },

    /// Set lid logo mode
    Logo {
        #[arg(value_enum)]
        mode: LogoMode,
    },

    /// Enable or disable battery care mode
    BatteryCare {
        #[arg(value_enum)]
        mode: BatteryCare,
    },

    /// Set lights always on mode
    LightsAlwaysOn {
        #[arg(value_enum)]
        mode: LightsAlwaysOn,
    },
}

#[derive(Subcommand)]
pub enum FanCommand {
    /// Set fan to automatic mode
    Auto,

    /// Set fan to manual mode with specific RPM
    Manual {
        /// Fan speed in RPM (2000-5000)
        #[arg(value_parser = clap::value_parser!(u16).range(2000..=5000))]
        rpm: u16,
    },

    /// Enable or disable max fan speed mode
    Max {
        #[arg(value_enum)]
        mode: MaxFanSpeedMode,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,

    /// Set default profile
    SetDefault {
        /// Profile name to set as default
        profile: String,
    },

    /// Clear cached device PID
    ClearCache,

    /// Show configuration file path
    Path,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SettingName {
    /// Performance mode
    Perf,
    /// CPU boost level
    Cpu,
    /// GPU boost level
    Gpu,
    /// Fan mode and speed
    Fan,
    /// Max fan speed mode
    MaxFan,
    /// Keyboard backlight brightness
    Keyboard,
    /// Lid logo mode
    Logo,
    /// Battery care mode
    BatteryCare,
    /// Lights always on mode
    LightsAlwaysOn,
}
