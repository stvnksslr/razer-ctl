use crate::config::ConfigManager;
use crate::error::{Error, Result};
use crate::settings::{DeviceState, Setting, SettingValue};
use librazer::{command, descriptor, device, types};
use log::debug;

/// Check if a Razer USB device is physically connected (Linux only).
/// This checks /sys directly, bypassing hidapi permissions.
#[cfg(target_os = "linux")]
fn razer_device_exists() -> bool {
    use std::fs;
    use std::path::Path;

    let usb_devices = Path::new("/sys/bus/usb/devices");
    if let Ok(entries) = fs::read_dir(usb_devices) {
        for entry in entries.flatten() {
            let vendor_path = entry.path().join("idVendor");
            if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                if vendor.trim().eq_ignore_ascii_case("1532") {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
fn razer_device_exists() -> bool {
    false
}

pub struct BladeDevice {
    inner: device::Device,
}

impl BladeDevice {
    pub fn detect() -> Result<Self> {
        let inner = device::Device::detect().map_err(|e| {
            let err_msg = e.to_string().to_lowercase();

            // Skip permission check if the error is about invalid arguments (protocol issue)
            if err_msg.contains("einval") || err_msg.contains("invalid argument") {
                return Error::DeviceNotFound;
            }

            // Check for permission-related errors
            if err_msg.contains("permission")
                || err_msg.contains("access denied")
                || err_msg.contains("operation not permitted")
            {
                return Error::PermissionDenied;
            }

            // On Linux, if device exists in /sys but hidapi can't see it, likely permissions
            if razer_device_exists() {
                return Error::PermissionDenied;
            }

            Error::DeviceNotFound
        })?;
        Ok(Self { inner })
    }

    pub fn detect_with_cache() -> Result<Self> {
        // Try to load config and use cached PID
        if let Ok(config_mgr) = ConfigManager::load() {
            if let Some(cached_pid) = config_mgr.get_cached_pid() {
                debug!("Trying cached PID: {:#06x}", cached_pid);
                if let Some(desc) = descriptor::SUPPORTED.iter().find(|d| d.pid == cached_pid) {
                    if let Ok(inner) = device::Device::new(desc.clone()) {
                        debug!("Successfully connected using cached PID");
                        return Ok(Self { inner });
                    }
                }
                debug!("Cached PID failed, falling back to full detection");
            }
        }

        // Fall back to full detection
        let device = Self::detect()?;

        // Cache the detected device
        if let Ok(mut config_mgr) = ConfigManager::load() {
            let _ = config_mgr.set_cached_device(device.pid(), device.name(), device.model());
        }

        Ok(device)
    }

    pub fn name(&self) -> &str {
        self.inner.info.name
    }

    pub fn model(&self) -> &str {
        self.inner.info.model_number_prefix
    }

    pub fn pid(&self) -> u16 {
        self.inner.info.pid
    }

    pub fn features(&self) -> &[&str] {
        self.inner.info.features
    }

    pub fn supports(&self, feature: &str) -> bool {
        self.inner.info.features.contains(&feature)
    }

    pub fn read_state(&self) -> Result<DeviceState> {
        let mut state = DeviceState::default();

        // Performance mode
        if let Ok((perf_mode, fan_mode)) = command::get_perf_mode(&self.inner) {
            state.perf_mode = Some(perf_mode);
            state.fan_mode = Some(fan_mode);

            if perf_mode == types::PerfMode::Custom {
                state.cpu_boost = command::get_cpu_boost(&self.inner).ok();
                state.gpu_boost = command::get_gpu_boost(&self.inner).ok();
            }

            if fan_mode == types::FanMode::Manual {
                state.fan_rpm = command::get_fan_rpm(&self.inner, types::FanZone::Zone1).ok();
            }
        }

        // Max fan speed mode
        state.max_fan_speed = command::get_max_fan_speed_mode(&self.inner).ok();

        // Keyboard brightness
        if self.supports("kbd-backlight") {
            state.keyboard_brightness = command::get_keyboard_brightness(&self.inner).ok();
        }

        // Battery care
        if self.supports("battery-care") {
            state.battery_care = command::get_battery_care(&self.inner).ok();
        }

        // Logo mode
        if self.supports("lid-logo") {
            state.logo_mode = command::get_logo_mode(&self.inner).ok();
        }

        // Lights always on
        if self.supports("lights-always-on") {
            state.lights_always_on = command::get_lights_always_on(&self.inner).ok();
        }

        Ok(state)
    }

    pub fn get_setting(&self, setting: Setting) -> Result<SettingValue> {
        match setting {
            Setting::PerfMode => {
                let (mode, fan_mode) = command::get_perf_mode(&self.inner)?;
                Ok(SettingValue::PerfMode { mode, fan_mode })
            }
            Setting::CpuBoost => {
                let boost = command::get_cpu_boost(&self.inner)?;
                Ok(SettingValue::CpuBoost(boost))
            }
            Setting::GpuBoost => {
                let boost = command::get_gpu_boost(&self.inner)?;
                Ok(SettingValue::GpuBoost(boost))
            }
            Setting::FanMode => {
                let (_, fan_mode) = command::get_perf_mode(&self.inner)?;
                let rpm = if fan_mode == types::FanMode::Manual {
                    Some(command::get_fan_rpm(&self.inner, types::FanZone::Zone1)?)
                } else {
                    None
                };
                Ok(SettingValue::Fan {
                    mode: fan_mode,
                    rpm,
                })
            }
            Setting::MaxFanSpeed => {
                let mode = command::get_max_fan_speed_mode(&self.inner)?;
                Ok(SettingValue::MaxFanSpeed(mode))
            }
            Setting::KeyboardBrightness => {
                if !self.supports("kbd-backlight") {
                    return Err(Error::FeatureNotSupported("kbd-backlight".to_string()));
                }
                let brightness = command::get_keyboard_brightness(&self.inner)?;
                Ok(SettingValue::KeyboardBrightness(brightness))
            }
            Setting::LogoMode => {
                if !self.supports("lid-logo") {
                    return Err(Error::FeatureNotSupported("lid-logo".to_string()));
                }
                let mode = command::get_logo_mode(&self.inner)?;
                Ok(SettingValue::LogoMode(mode))
            }
            Setting::BatteryCare => {
                if !self.supports("battery-care") {
                    return Err(Error::FeatureNotSupported("battery-care".to_string()));
                }
                let care = command::get_battery_care(&self.inner)?;
                Ok(SettingValue::BatteryCare(care))
            }
            Setting::LightsAlwaysOn => {
                if !self.supports("lights-always-on") {
                    return Err(Error::FeatureNotSupported("lights-always-on".to_string()));
                }
                let lights = command::get_lights_always_on(&self.inner)?;
                Ok(SettingValue::LightsAlwaysOn(lights))
            }
        }
    }

    pub fn apply_setting(&self, value: SettingValue) -> Result<()> {
        match value {
            SettingValue::PerfMode { mode, .. } => {
                command::set_perf_mode(&self.inner, mode)?;
            }
            SettingValue::CpuBoost(boost) => {
                command::set_cpu_boost(&self.inner, boost)?;
            }
            SettingValue::GpuBoost(boost) => {
                command::set_gpu_boost(&self.inner, boost)?;
            }
            SettingValue::Fan { mode, rpm } => {
                command::set_fan_mode(&self.inner, mode)?;
                if let Some(rpm) = rpm {
                    command::set_fan_rpm(&self.inner, rpm)?;
                }
            }
            SettingValue::MaxFanSpeed(mode) => {
                command::set_max_fan_speed_mode(&self.inner, mode)?;
            }
            SettingValue::KeyboardBrightness(brightness) => {
                if !self.supports("kbd-backlight") {
                    return Err(Error::FeatureNotSupported("kbd-backlight".to_string()));
                }
                command::set_keyboard_brightness(&self.inner, brightness)?;
            }
            SettingValue::LogoMode(mode) => {
                if !self.supports("lid-logo") {
                    return Err(Error::FeatureNotSupported("lid-logo".to_string()));
                }
                command::set_logo_mode(&self.inner, mode)?;
            }
            SettingValue::BatteryCare(care) => {
                if !self.supports("battery-care") {
                    return Err(Error::FeatureNotSupported("battery-care".to_string()));
                }
                command::set_battery_care(&self.inner, care)?;
            }
            SettingValue::LightsAlwaysOn(lights) => {
                if !self.supports("lights-always-on") {
                    return Err(Error::FeatureNotSupported("lights-always-on".to_string()));
                }
                command::set_lights_always_on(&self.inner, lights)?;
            }
        }
        Ok(())
    }
}
