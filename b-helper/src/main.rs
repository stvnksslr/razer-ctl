use iced::{
    executor, theme, Alignment, Application, Color, Command, Element, Length, Settings,
    Theme,
};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, row, 
    scrollable, slider, text, vertical_space,
};
use iced::window;
use std::process::Command as ProcessCommand;
use anyhow::Result;

// Import required types from librazer
// These would normally be part of librazer dependency
// Including simplified versions here for the sake of demonstration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfMode {
    Balanced,
    Silent,
    Custom,
}

impl std::fmt::Display for PerfMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerfMode::Balanced => write!(f, "Balanced"),
            PerfMode::Silent => write!(f, "Silent"),
            PerfMode::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBoost {
    Low,
    Medium,
    High,
    Boost,
    Overclock,
}

impl std::fmt::Display for CpuBoost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuBoost::Low => write!(f, "Low"),
            CpuBoost::Medium => write!(f, "Medium"),
            CpuBoost::High => write!(f, "High"),
            CpuBoost::Boost => write!(f, "Boost"),
            CpuBoost::Overclock => write!(f, "Overclock"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBoost {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for GpuBoost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuBoost::Low => write!(f, "Eco"),
            GpuBoost::Medium => write!(f, "Standard"),
            GpuBoost::High => write!(f, "Ultimate"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoMode {
    Off,
    Breathing,
    Static,
}

impl std::fmt::Display for LogoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogoMode::Off => write!(f, "Off"),
            LogoMode::Breathing => write!(f, "Breathing"),
            LogoMode::Static => write!(f, "Static"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightsAlwaysOn {
    Enable,
    Disable,
}

impl std::fmt::Display for LightsAlwaysOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightsAlwaysOn::Enable => write!(f, "Enable"),
            LightsAlwaysOn::Disable => write!(f, "Disable"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryCare {
    Enable,
    Disable,
}

impl std::fmt::Display for BatteryCare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryCare::Enable => write!(f, "Enable"),
            BatteryCare::Disable => write!(f, "Disable"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxFanSpeedMode {
    Enable,
    Disable,
}

// Feature tracker to know what features are supported on the current device
#[derive(Debug, Clone)]
struct SupportedFeatures {
    battery_care: bool,
    fan: bool,
    kbd_backlight: bool,
    lid_logo: bool,
    lights_always_on: bool,
    perf: bool,
}

impl Default for SupportedFeatures {
    fn default() -> Self {
        Self {
            battery_care: true,
            fan: true,
            kbd_backlight: true,
            lid_logo: true,
            lights_always_on: true,
            perf: true,
        }
    }
}

// Main application state
#[derive(Debug, Clone)]
struct RazerUI {
    // Device info
    model_name: String,
    device_connected: bool,
    supported_features: SupportedFeatures,
    
    // Performance settings
    perf_mode: PerfMode,
    fan_mode: FanMode,
    cpu_boost: CpuBoost,
    gpu_boost: GpuBoost,
    max_fan_speed: MaxFanSpeedMode,
    fan_rpm: u16,
    
    // Lighting settings
    kbd_brightness: u8,
    logo_mode: LogoMode,
    lights_always_on: LightsAlwaysOn,
    
    // Battery settings
    battery_care: BatteryCare,
    battery_limit: u8,
    battery_percentage: f32,
    battery_charging: bool,
    run_on_startup: bool,
    
    // System info
    cpu_temp: f32,
    fan_rpm_current: u16,
}

#[derive(Debug, Clone)]
enum Message {
    PerfModeChanged(PerfMode),
    FanModeChanged(FanMode),
    CpuBoostChanged(CpuBoost),
    GpuBoostChanged(GpuBoost),
    MaxFanSpeedChanged(MaxFanSpeedMode),
    FanRpmChanged(u16),
    
    KbdBrightnessChanged(u8),
    LogoModeChanged(LogoMode),
    LightsAlwaysOnChanged(LightsAlwaysOn),
    
    BatteryCareChanged(BatteryCare),
    BatteryLimitChanged(u8),
    RunOnStartupChanged(bool),
    
    RefreshStatus,
    Exit,
}

fn execute_razer_cli(args: &[&str]) -> Result<String> {
    let output = ProcessCommand::new("razer-cli")
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute razer-cli: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

impl Application for RazerUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let default_app = Self {
            model_name: String::from("Razer Laptop Control"),
            device_connected: false,
            supported_features: SupportedFeatures::default(),
            
            perf_mode: PerfMode::Balanced,
            fan_mode: FanMode::Auto,
            cpu_boost: CpuBoost::Medium,
            gpu_boost: GpuBoost::Medium,
            max_fan_speed: MaxFanSpeedMode::Disable,
            fan_rpm: 2000,
            
            kbd_brightness: 128,
            logo_mode: LogoMode::Static,
            lights_always_on: LightsAlwaysOn::Disable,
            
            battery_care: BatteryCare::Enable,
            battery_limit: 80,
            battery_percentage: 75.0,
            battery_charging: false,
            run_on_startup: true,
            
            cpu_temp: 32.0,
            fan_rpm_current: 0,
        };
        
        (default_app, Command::perform(async {}, |_| Message::RefreshStatus))
    }

    fn title(&self) -> String {
        format!("Razer Control — {}", self.model_name)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PerfModeChanged(mode) => {
                self.perf_mode = mode;
                if let Err(e) = execute_razer_cli(&["auto", "perf", "mode", &mode.to_string()]) {
                    eprintln!("Failed to set performance mode: {}", e);
                }
            }
            Message::FanModeChanged(mode) => {
                self.fan_mode = mode;
                let cmd = match mode {
                    FanMode::Auto => "auto",
                    FanMode::Manual => "manual",
                };
                if let Err(e) = execute_razer_cli(&["auto", "fan", cmd]) {
                    eprintln!("Failed to set fan mode: {}", e);
                }
            }
            Message::CpuBoostChanged(boost) => {
                self.cpu_boost = boost;
                if let Err(e) = execute_razer_cli(&["auto", "perf", "cpu", &boost.to_string()]) {
                    eprintln!("Failed to set CPU boost: {}", e);
                }
            }
            Message::GpuBoostChanged(boost) => {
                self.gpu_boost = boost;
                if let Err(e) = execute_razer_cli(&["auto", "perf", "gpu", &boost.to_string()]) {
                    eprintln!("Failed to set GPU boost: {}", e);
                }
            }
            Message::MaxFanSpeedChanged(mode) => {
                self.max_fan_speed = mode;
                let value = match mode {
                    MaxFanSpeedMode::Enable => "enable",
                    MaxFanSpeedMode::Disable => "disable",
                };
                if let Err(e) = execute_razer_cli(&["auto", "fan", "max", value]) {
                    eprintln!("Failed to set max fan speed: {}", e);
                }
            }
            Message::FanRpmChanged(rpm) => {
                self.fan_rpm = rpm;
                if let Err(e) = execute_razer_cli(&["auto", "fan", "rpm", &rpm.to_string()]) {
                    eprintln!("Failed to set fan RPM: {}", e);
                }
            }
            Message::KbdBrightnessChanged(brightness) => {
                self.kbd_brightness = brightness;
                if let Err(e) = execute_razer_cli(&["auto", "kbd-backlight", &brightness.to_string()]) {
                    eprintln!("Failed to set keyboard brightness: {}", e);
                }
            }
            Message::LogoModeChanged(mode) => {
                self.logo_mode = mode;
                if let Err(e) = execute_razer_cli(&["auto", "lid-logo", &mode.to_string()]) {
                    eprintln!("Failed to set logo mode: {}", e);
                }
            }
            Message::LightsAlwaysOnChanged(mode) => {
                self.lights_always_on = mode;
                let value = match mode {
                    LightsAlwaysOn::Enable => "enable",
                    LightsAlwaysOn::Disable => "disable",
                };
                if let Err(e) = execute_razer_cli(&["auto", "lights-always-on", value]) {
                    eprintln!("Failed to set lights always on: {}", e);
                }
            }
            Message::BatteryCareChanged(mode) => {
                self.battery_care = mode;
                let value = match mode {
                    BatteryCare::Enable => "enable",
                    BatteryCare::Disable => "disable",
                };
                if let Err(e) = execute_razer_cli(&["auto", "battery-care", value]) {
                    eprintln!("Failed to set battery care: {}", e);
                }
            }
            Message::BatteryLimitChanged(limit) => {
                self.battery_limit = limit;
                // Note: This isn't directly supported by razer-cli
                // This would require custom implementation
            }
            Message::RunOnStartupChanged(enabled) => {
                self.run_on_startup = enabled;
                // This would need to be implemented by creating/removing startup scripts
            }
            Message::RefreshStatus => {
                // This would call razer-cli info and update the UI state
                if let Ok(info) = execute_razer_cli(&["auto", "info"]) {
                    self.device_connected = true;
                    
                    // Parse model name
                    if let Some(line) = info.lines().find(|l| l.contains("Device:")) {
                        if let Some(name) = line.split(':').nth(1) {
                            self.model_name = name.trim().to_string();
                        }
                    }
                    
                    // This would parse all the settings from the info command
                    // and update the UI state accordingly
                }
                
                // Refresh system info (would need additional system commands)
                // For demo, we'll just update with random values
                self.cpu_temp = 30.0 + (rand::random::<f32>() * 10.0);
                self.fan_rpm_current = (2000 + (rand::random::<u16>() % 2000)) / 100 * 100;
                self.battery_percentage = 70.0 + (rand::random::<f32>() * 20.0);
            }
            Message::Exit => {
                std::process::exit(0);
            }
        }
        
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text(format!("{} — {}", self.model_name, if self.device_connected { "Connected" } else { "Disconnected" }))
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center);
        
        // Performance Mode Section
        let perf_title = row![
            text(format!("Mode: {}", self.perf_mode)).size(16),
            horizontal_space(Length::Fill),
            text(format!("CPU: {}°C", self.cpu_temp.round())).size(16),
            horizontal_space(10),
            text(format!("Fan: {}RPM", self.fan_rpm_current)).size(16),
        ].width(Length::Fill);
        
        let perf_buttons = row![
            perf_mode_button("Silent", self.perf_mode == PerfMode::Silent, PerfMode::Silent),
            perf_mode_button("Balanced", self.perf_mode == PerfMode::Balanced, PerfMode::Balanced),
            perf_mode_button("Custom", self.perf_mode == PerfMode::Custom, PerfMode::Custom),
        ].spacing(10).width(Length::Fill);
        
        // GPU Mode Section
        let gpu_title = row![
            text("GPU Mode").size(16),
            horizontal_space(Length::Fill),
            text(format!("GPU Fan: {}RPM", self.fan_rpm_current)).size(16),
        ].width(Length::Fill);
        
        let gpu_buttons = row![
            gpu_mode_button("Eco", self.gpu_boost == GpuBoost::Low, GpuBoost::Low),
            gpu_mode_button("Standard", self.gpu_boost == GpuBoost::Medium, GpuBoost::Medium),
            gpu_mode_button("Ultimate", self.gpu_boost == GpuBoost::High, GpuBoost::High),
        ].spacing(10).width(Length::Fill);
        
        // Fan Control Section
        let fan_title = text("Fan Control").size(16);
        
        let fan_controls = row![
            button(text("Auto").horizontal_alignment(iced::alignment::Horizontal::Center))
                .on_press(Message::FanModeChanged(FanMode::Auto))
                .width(Length::Fill)
                .style(if self.fan_mode == FanMode::Auto {
                    theme::Button::Primary
                } else {
                    theme::Button::Secondary
                }),
            button(text("Manual").horizontal_alignment(iced::alignment::Horizontal::Center))
                .on_press(Message::FanModeChanged(FanMode::Manual))
                .width(Length::Fill)
                .style(if self.fan_mode == FanMode::Manual {
                    theme::Button::Primary
                } else {
                    theme::Button::Secondary
                }),
        ].spacing(10).width(Length::Fill);
        
        let fan_rpm_control = if self.fan_mode == FanMode::Manual {
            row![
                text(format!("Fan RPM: {}", self.fan_rpm)),
                slider(2000..=5000, self.fan_rpm, |rpm| Message::FanRpmChanged(rpm))
                    .step(100)
                    .width(Length::Fill),
            ].spacing(10).width(Length::Fill)
        } else {
            row![].width(Length::Fill)
        };
        
        let max_fan_checkbox = checkbox(
            "Max Fan Speed",
            self.max_fan_speed == MaxFanSpeedMode::Enable,
            |checked| {
                Message::MaxFanSpeedChanged(if checked {
                    MaxFanSpeedMode::Enable
                } else {
                    MaxFanSpeedMode::Disable
                })
            },
        );
        
        // Keyboard Backlight Section
        let kbd_title = row![
            text("Keyboard Backlight").size(16)
        ].width(Length::Fill);
        
        let kbd_brightness_control = row![
            text(format!("Brightness: {}", self.kbd_brightness)),
            slider(0..=255, self.kbd_brightness, Message::KbdBrightnessChanged)
                .step(10)
                .width(Length::Fill),
        ].spacing(10).width(Length::Fill);
        
        // Logo Control Section (only if supported)
        let logo_section = if self.supported_features.lid_logo {
            let logo_modes = vec![LogoMode::Off, LogoMode::Static, LogoMode::Breathing];
            let logo_control = pick_list(
                logo_modes,
                Some(self.logo_mode),
                Message::LogoModeChanged
            ).width(120);
            
            column![
                text("Lid Logo").size(16),
                row![
                    text("Mode:"),
                    horizontal_space(10),
                    logo_control,
                ].spacing(10),
            ].spacing(10)
        } else {
            column![]
        };
        
        // Lights Always On Section
        let lights_section = if self.supported_features.lights_always_on {
            let lights_options = vec![LightsAlwaysOn::Enable, LightsAlwaysOn::Disable];
            let lights_control = pick_list(
                lights_options, 
                Some(self.lights_always_on),
                Message::LightsAlwaysOnChanged
            ).width(120);
            
            column![
                text("Lights Always On").size(16),
                row![
                    text("Mode:"),
                    horizontal_space(10),
                    lights_control,
                ].spacing(10),
            ].spacing(10)
        } else {
            column![]
        };
        
        // Battery Care Section
        let battery_title = row![
            text(format!("Battery Charge Limit: {}%", self.battery_limit)).size(16),
            horizontal_space(Length::Fill),
            text(format!("{}: {:.1}W", 
                if self.battery_charging { "Charging" } else { "Discharging" }, 
                5.0)).size(16),
        ].width(Length::Fill);
        
        let battery_slider = slider(50..=100, self.battery_limit, Message::BatteryLimitChanged)
            .step(5)
            .width(Length::Fill);
        
        let battery_percentage = row![
            text(format!("Charge: {:.1}%", self.battery_percentage)),
        ].width(Length::Fill);
        
        let battery_care_checkbox = if self.supported_features.battery_care {
            checkbox(
                "Enable Battery Care",
                self.battery_care == BatteryCare::Enable,
                |checked| {
                    Message::BatteryCareChanged(if checked {
                        BatteryCare::Enable
                    } else {
                        BatteryCare::Disable
                    })
                },
            )
        } else {
            checkbox(
                "Battery Care (Not Supported)", 
                false, 
                |_| Message::BatteryCareChanged(BatteryCare::Disable)
            )
        };
        
        let startup_checkbox = checkbox(
            "Run on Startup",
            self.run_on_startup,
            Message::RunOnStartupChanged,
        );
        
        // Footer
        let footer = row![
            text("Version: 0.1.0"),
            horizontal_space(Length::Fill),
            button(text("Updates")).width(100),
            button(text("Quit"))
                .on_press(Message::Exit)
                .width(100),
        ].spacing(10).width(Length::Fill);
        
        // Build the main content
        let content = column![
            title,
            vertical_space(10),
            
            // Performance Section
            perf_title,
            perf_buttons,
            vertical_space(20),
            
            // GPU Section
            gpu_title,
            gpu_buttons,
            vertical_space(20),
            
            // Fan Control
            fan_title,
            fan_controls,
            fan_rpm_control,
            max_fan_checkbox,
            vertical_space(20),
            
            // Keyboard Backlight
            kbd_title,
            kbd_brightness_control,
            vertical_space(20),
            
            // Logo Control
            logo_section,
            vertical_space(10),
            
            // Lights Always On
            lights_section,
            vertical_space(20),
            
            // Battery Section
            battery_title,
            battery_slider,
            battery_percentage,
            battery_care_checkbox,
            startup_checkbox,
            vertical_space(20),
            
            horizontal_rule(1),
            vertical_space(10),
            
            // Footer
            footer,
        ]
        .spacing(8)
        .padding(20);
        
        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(theme::Container::Box)
            .into()
    }
}

fn perf_mode_button<'a>(
    label: &str,
    selected: bool,
    perf_mode: PerfMode,
) -> Element<'a, Message> {
    let btn = button(
        container(
            column![
                text(match label {
                    "Silent" => "🔇",
                    "Balanced" => "⚖️",
                    "Custom" => "🚀",
                    _ => "⚙️",
                }).size(24).horizontal_alignment(iced::alignment::Horizontal::Center),
                text(label).horizontal_alignment(iced::alignment::Horizontal::Center),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .padding(10)
        .width(Length::Fill)
        .center_x()
    )
    .width(Length::Fill)
    .style(if selected {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    });
    
    if selected {
        btn.into()
    } else {
        btn.on_press(Message::PerfModeChanged(perf_mode)).into()
    }
}

fn gpu_mode_button<'a>(
    label: &str,
    selected: bool,
    gpu_boost: GpuBoost,
) -> Element<'a, Message> {
    let btn = button(
        container(
            column![
                text(match label {
                    "Eco" => "🍃",
                    "Standard" => "⚡",
                    "Ultimate" => "🔥",
                    _ => "⚙️",
                }).size(24).horizontal_alignment(iced::alignment::Horizontal::Center),
                text(label).horizontal_alignment(iced::alignment::Horizontal::Center),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .padding(10)
        .width(Length::Fill)
        .center_x()
    )
    .width(Length::Fill)
    .style(if selected {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    });
    
    if selected {
        btn.into()
    } else {
        btn.on_press(Message::GpuBoostChanged(gpu_boost)).into()
    }
}

fn main() -> Result<(), iced::Error> {
    let settings = Settings {
        window: window::Settings {
            size: (500, 650),
            ..window::Settings::default()
        },
        ..Settings::default()
    };
    
    RazerUI::run(settings)
}