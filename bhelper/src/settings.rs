use librazer::types::{
    BatteryCare, CpuBoost, FanMode, GpuBoost, LightsAlwaysOn, LogoMode, MaxFanSpeedMode, PerfMode,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub enum Setting {
    PerfMode,
    CpuBoost,
    GpuBoost,
    FanMode,
    MaxFanSpeed,
    KeyboardBrightness,
    LogoMode,
    BatteryCare,
    LightsAlwaysOn,
}

#[derive(Clone, Debug)]
pub enum SettingValue {
    PerfMode { mode: PerfMode, fan_mode: FanMode },
    CpuBoost(CpuBoost),
    GpuBoost(GpuBoost),
    Fan { mode: FanMode, rpm: Option<u16> },
    MaxFanSpeed(MaxFanSpeedMode),
    KeyboardBrightness(u8),
    LogoMode(LogoMode),
    BatteryCare(BatteryCare),
    LightsAlwaysOn(LightsAlwaysOn),
}

#[derive(Clone, Debug, Default)]
pub struct DeviceState {
    pub perf_mode: Option<PerfMode>,
    pub fan_mode: Option<FanMode>,
    pub cpu_boost: Option<CpuBoost>,
    pub gpu_boost: Option<GpuBoost>,
    pub fan_rpm: Option<u16>,
    pub max_fan_speed: Option<MaxFanSpeedMode>,
    pub keyboard_brightness: Option<u8>,
    pub logo_mode: Option<LogoMode>,
    pub battery_care: Option<BatteryCare>,
    pub lights_always_on: Option<LightsAlwaysOn>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonDeviceState {
    pub perf_mode: Option<String>,
    pub fan_mode: Option<String>,
    pub cpu_boost: Option<String>,
    pub gpu_boost: Option<String>,
    pub fan_rpm: Option<u16>,
    pub max_fan_speed: Option<String>,
    pub keyboard_brightness: Option<u8>,
    pub logo_mode: Option<String>,
    pub battery_care: Option<String>,
    pub lights_always_on: Option<String>,
}

impl From<&DeviceState> for JsonDeviceState {
    fn from(state: &DeviceState) -> Self {
        Self {
            perf_mode: state.perf_mode.map(|m| format!("{:?}", m)),
            fan_mode: state.fan_mode.map(|m| format!("{:?}", m)),
            cpu_boost: state.cpu_boost.map(|m| format!("{:?}", m)),
            gpu_boost: state.gpu_boost.map(|m| format!("{:?}", m)),
            fan_rpm: state.fan_rpm,
            max_fan_speed: state.max_fan_speed.map(|m| format!("{:?}", m)),
            keyboard_brightness: state.keyboard_brightness,
            logo_mode: state.logo_mode.map(|m| format!("{:?}", m)),
            battery_care: state.battery_care.map(|m| format!("{:?}", m)),
            lights_always_on: state.lights_always_on.map(|m| format!("{:?}", m)),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonDeviceInfo {
    pub name: String,
    pub model: String,
    pub pid: String,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonSettingValue {
    pub setting: String,
    pub value: String,
}

impl std::fmt::Display for SettingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingValue::PerfMode { mode, fan_mode } => {
                write!(f, "{:?} (Fan: {:?})", mode, fan_mode)
            }
            SettingValue::CpuBoost(boost) => write!(f, "{:?}", boost),
            SettingValue::GpuBoost(boost) => write!(f, "{:?}", boost),
            SettingValue::Fan { mode, rpm } => match (mode, rpm) {
                (FanMode::Auto, _) => write!(f, "Auto"),
                (FanMode::Manual, Some(rpm)) => write!(f, "Manual @ {} RPM", rpm),
                (FanMode::Manual, None) => write!(f, "Manual"),
            },
            SettingValue::MaxFanSpeed(mode) => write!(f, "{:?}", mode),
            SettingValue::KeyboardBrightness(b) => write!(f, "{}", b),
            SettingValue::LogoMode(mode) => write!(f, "{:?}", mode),
            SettingValue::BatteryCare(care) => write!(f, "{:?}", care),
            SettingValue::LightsAlwaysOn(lights) => write!(f, "{:?}", lights),
        }
    }
}
