use crate::device::Device;
use crate::packet::Packet;
use crate::types::{
    BatteryCare, Cluster, CpuBoost, FanMode, FanZone, GpuBoost, LightsAlwaysOn, LogoMode,
    MaxFanSpeedMode, PerfMode,
};
use anyhow::{bail, ensure, Result};
use log::debug;

// USB HID command codes - see data/README.md for protocol details
mod cmd {
    // Performance mode commands
    pub const SET_PERF_MODE: u16 = 0x0d02;
    pub const GET_PERF_MODE: u16 = 0x0d82;
    pub const SET_BOOST: u16 = 0x0d07;
    pub const GET_BOOST: u16 = 0x0d87;

    // Fan commands
    pub const SET_FAN_RPM: u16 = 0x0d01;
    pub const GET_FAN_RPM: u16 = 0x0d81;
    pub const SET_MAX_FAN_SPEED: u16 = 0x070f;
    pub const GET_MAX_FAN_SPEED: u16 = 0x078f;

    // Logo commands
    pub const SET_LOGO_POWER: u16 = 0x0300;
    pub const GET_LOGO_POWER: u16 = 0x0380;
    pub const SET_LOGO_MODE: u16 = 0x0302;
    pub const GET_LOGO_MODE: u16 = 0x0382;

    // Keyboard commands
    pub const SET_KBD_BRIGHTNESS: u16 = 0x0303;
    pub const GET_KBD_BRIGHTNESS: u16 = 0x0383;

    // Lights always on
    pub const SET_LIGHTS_ALWAYS_ON: u16 = 0x0004;
    pub const GET_LIGHTS_ALWAYS_ON: u16 = 0x0084;

    // Battery care
    pub const SET_BATTERY_CARE: u16 = 0x0712;
    pub const GET_BATTERY_CARE: u16 = 0x0792;
}

fn send_command(device: &Device, command: u16, args: &[u8]) -> Result<Packet> {
    let response = device.send(Packet::new(command, args))?;
    ensure!(response.get_args().starts_with(args));
    Ok(response)
}

fn set_perf_mode_internal(device: &Device, perf_mode: PerfMode, fan_mode: FanMode) -> Result<()> {
    if (fan_mode == FanMode::Manual) && (perf_mode != PerfMode::Balanced) {
        bail!("{:?} allowed only in {:?}", fan_mode, PerfMode::Balanced);
    }

    [1, 2].into_iter().try_for_each(|zone| {
        send_command(
            device,
            cmd::SET_PERF_MODE,
            &[0x01, zone, perf_mode as u8, fan_mode as u8],
        )
        .map(|_| ())
    })
}

fn set_boost_internal(device: &Device, cluster: Cluster, boost: u8) -> Result<()> {
    let args = &[0, cluster as u8, boost];
    ensure!(
        get_perf_mode(device)? == (PerfMode::Custom, FanMode::Auto),
        "Performance mode must be {:?}",
        PerfMode::Custom
    );
    ensure!(device
        .send(Packet::new(cmd::SET_BOOST, args))?
        .get_args()
        .starts_with(args));
    Ok(())
}

fn get_boost_internal(device: &Device, cluster: Cluster) -> Result<u8> {
    let response = device.send(Packet::new(cmd::GET_BOOST, &[0, cluster as u8, 0]))?;
    ensure!(response.get_args()[1] == cluster as u8);
    Ok(response.get_args()[2])
}

/// Sets the laptop's performance mode (Silent, Balanced, or Custom).
///
/// Fan mode is automatically set to Auto. Use [`set_fan_mode`] to switch to manual fan control.
pub fn set_perf_mode(device: &Device, perf_mode: PerfMode) -> Result<()> {
    set_perf_mode_internal(device, perf_mode, FanMode::Auto)
}

/// Gets the current performance mode and fan mode.
///
/// Queries both thermal zones and ensures they match.
pub fn get_perf_mode(device: &Device) -> Result<(PerfMode, FanMode)> {
    let results: Vec<_> = [1, 2]
        .into_iter()
        .map(|zone| {
            let response = device.send(Packet::new(cmd::GET_PERF_MODE, &[0, zone, 0, 0]))?;
            Ok((
                PerfMode::try_from(response.get_args()[2])?,
                FanMode::try_from(response.get_args()[3])?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    ensure!(
        results[0] == results[1],
        "Modes do not match between zones: {:?} vs {:?}",
        results[0],
        results[1]
    );

    Ok(results[0])
}

/// Sets the CPU boost level. Requires Custom performance mode.
pub fn set_cpu_boost(device: &Device, boost: CpuBoost) -> Result<()> {
    set_boost_internal(device, Cluster::Cpu, boost as u8)
}

/// Sets the GPU boost level. Requires Custom performance mode.
pub fn set_gpu_boost(device: &Device, boost: GpuBoost) -> Result<()> {
    set_boost_internal(device, Cluster::Gpu, boost as u8)
}

/// Gets the current CPU boost level.
pub fn get_cpu_boost(device: &Device) -> Result<CpuBoost> {
    CpuBoost::try_from(get_boost_internal(device, Cluster::Cpu)?)
}

/// Gets the current GPU boost level.
pub fn get_gpu_boost(device: &Device) -> Result<GpuBoost> {
    GpuBoost::try_from(get_boost_internal(device, Cluster::Gpu)?)
}

/// Sets the fan speed in RPM. Valid range is 2000-5000.
///
/// Requires Balanced performance mode with Manual fan mode.
pub fn set_fan_rpm(device: &Device, rpm: u16) -> Result<()> {
    ensure!((2000..=5000).contains(&rpm));
    ensure!(
        get_perf_mode(device)? == (PerfMode::Balanced, FanMode::Manual),
        "Performance mode must be {:?} and fan mode must be {:?}",
        PerfMode::Balanced,
        FanMode::Manual
    );
    [FanZone::Zone1, FanZone::Zone2]
        .into_iter()
        .try_for_each(|zone| {
            send_command(
                device,
                cmd::SET_FAN_RPM,
                &[0, zone as u8, (rpm / 100) as u8],
            )
            .map(|_| ())
        })
}

/// Gets the current fan RPM for the specified zone.
pub fn get_fan_rpm(device: &Device, fan_zone: FanZone) -> Result<u16> {
    let response = device.send(Packet::new(cmd::GET_FAN_RPM, &[0, fan_zone as u8, 0]))?;
    ensure!(response.get_args()[1] == fan_zone as u8);
    Ok(response.get_args()[2] as u16 * 100)
}

/// Enables or disables max fan speed mode. Requires Custom performance mode.
pub fn set_max_fan_speed_mode(device: &Device, mode: MaxFanSpeedMode) -> Result<()> {
    ensure!(
        get_perf_mode(device)?.0 == PerfMode::Custom,
        "Performance mode must be {:?}",
        PerfMode::Custom
    );
    send_command(device, cmd::SET_MAX_FAN_SPEED, &[mode as u8]).map(|_| ())
}

/// Gets the current max fan speed mode setting.
pub fn get_max_fan_speed_mode(device: &Device) -> Result<MaxFanSpeedMode> {
    device
        .send(Packet::new(cmd::GET_MAX_FAN_SPEED, &[0]))?
        .get_args()[0]
        .try_into()
}

/// Sets the fan mode to Auto or Manual. Requires Balanced performance mode.
pub fn set_fan_mode(device: &Device, mode: FanMode) -> Result<()> {
    ensure!(
        get_perf_mode(device)?.0 == PerfMode::Balanced,
        "Performance mode must be {:?}",
        PerfMode::Balanced
    );
    set_perf_mode_internal(device, PerfMode::Balanced, mode)
}

/// Sends a custom USB HID command to the device.
///
/// # Warning
/// Use at your own risk. Incorrect commands may cause unexpected behavior.
pub fn custom_command(device: &Device, command: u16, args: &[u8]) -> Result<()> {
    let report = Packet::new(command, args);
    debug!("Report   {:?}", report);
    let response = device.send(report)?;
    debug!("Response {:?}", response);
    Ok(())
}

fn set_logo_power(device: &Device, mode: LogoMode) -> Result<Packet> {
    match mode {
        LogoMode::Off => send_command(device, cmd::SET_LOGO_POWER, &[1, 4, 0]),
        LogoMode::Static | LogoMode::Breathing => {
            send_command(device, cmd::SET_LOGO_POWER, &[1, 4, 1])
        }
    }
}

fn set_logo_mode_internal(device: &Device, mode: LogoMode) -> Result<Packet> {
    match mode {
        LogoMode::Static => send_command(device, cmd::SET_LOGO_MODE, &[1, 4, 0]),
        LogoMode::Breathing => send_command(device, cmd::SET_LOGO_MODE, &[1, 4, 2]),
        _ => bail!("Invalid logo mode"),
    }
}

fn get_logo_power(device: &Device) -> Result<bool> {
    match device
        .send(Packet::new(cmd::GET_LOGO_POWER, &[1, 4, 0]))?
        .get_args()[2]
    {
        0 => Ok(false),
        1 => Ok(true),
        _ => bail!("Invalid logo power state"),
    }
}

fn get_logo_mode_internal(device: &Device) -> Result<LogoMode> {
    match device
        .send(Packet::new(cmd::GET_LOGO_MODE, &[1, 4, 0]))?
        .get_args()[2]
    {
        0 => Ok(LogoMode::Static),
        2 => Ok(LogoMode::Breathing),
        _ => bail!("Invalid logo power state"),
    }
}

/// Gets the current lid logo mode (Off, Static, or Breathing).
pub fn get_logo_mode(device: &Device) -> Result<LogoMode> {
    let power = get_logo_power(device)?;
    match power {
        true => get_logo_mode_internal(device),
        false => Ok(LogoMode::Off),
    }
}

/// Sets the lid logo mode (Off, Static, or Breathing).
pub fn set_logo_mode(device: &Device, mode: LogoMode) -> Result<()> {
    if mode != LogoMode::Off {
        set_logo_mode_internal(device, mode)?;
    }
    set_logo_power(device, mode)?;
    Ok(())
}

/// Gets the current keyboard backlight brightness (0-255).
pub fn get_keyboard_brightness(device: &Device) -> Result<u8> {
    let response = device.send(Packet::new(cmd::GET_KBD_BRIGHTNESS, &[1, 5, 0]))?;
    ensure!(response.get_args()[1] == 5);
    Ok(response.get_args()[2])
}

/// Sets the keyboard backlight brightness (0-255).
pub fn set_keyboard_brightness(device: &Device, brightness: u8) -> Result<()> {
    let args = &[1, 5, brightness];
    ensure!(device
        .send(Packet::new(cmd::SET_KBD_BRIGHTNESS, args))?
        .get_args()
        .starts_with(args));
    Ok(())
}

/// Gets whether lights stay on when the laptop is closed/sleeping.
pub fn get_lights_always_on(device: &Device) -> Result<LightsAlwaysOn> {
    device
        .send(Packet::new(cmd::GET_LIGHTS_ALWAYS_ON, &[0, 0]))?
        .get_args()[0]
        .try_into()
}

/// Sets whether lights stay on when the laptop is closed/sleeping.
pub fn set_lights_always_on(device: &Device, lights_always_on: LightsAlwaysOn) -> Result<()> {
    let args = &[lights_always_on as u8, 0];
    ensure!(device
        .send(Packet::new(cmd::SET_LIGHTS_ALWAYS_ON, args))?
        .get_args()
        .starts_with(args));
    Ok(())
}

/// Gets the battery care mode (limits charging to 80% to extend battery life).
pub fn get_battery_care(device: &Device) -> Result<BatteryCare> {
    device
        .send(Packet::new(cmd::GET_BATTERY_CARE, &[0]))?
        .get_args()[0]
        .try_into()
}

/// Sets the battery care mode (limits charging to 80% to extend battery life).
pub fn set_battery_care(device: &Device, mode: BatteryCare) -> Result<()> {
    let args = &[mode as u8];
    ensure!(device
        .send(Packet::new(cmd::SET_BATTERY_CARE, args))?
        .get_args()
        .starts_with(args));
    Ok(())
}
