use anyhow::{bail, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

#[derive(Clone, Copy)]
pub enum Cluster {
    Cpu = 0x01,
    Gpu = 0x02,
}

#[derive(Clone, Copy)]
pub enum FanZone {
    Zone1 = 0x01,
    Zone2 = 0x02,
}

#[derive(Clone, Copy, Debug, PartialEq, EnumIter, ValueEnum)]
pub enum PerfMode {
    Balanced = 0,
    Silent = 5,
    Custom = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, ValueEnum)]
pub enum MaxFanSpeedMode {
    Enable = 2,
    Disable = 0,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FanMode {
    Auto = 0,
    Manual = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, ValueEnum)]
pub enum CpuBoost {
    Low = 0,
    Medium = 1,
    High = 2,
    Boost = 3,
    Overclock = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, ValueEnum)]
pub enum GpuBoost {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, EnumString, ValueEnum)]
pub enum LogoMode {
    Off,
    Breathing,
    Static,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumString, ValueEnum)]
pub enum LightsAlwaysOn {
    Enable = 0x03,
    Disable = 0x00,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum BatteryCare {
    Disable = 0x50,
    Enable = 0xd0,
}

impl TryFrom<u8> for GpuBoost {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::Medium),
            2 => Ok(Self::High),
            _ => bail!("Failed to convert {} to GpuBoost", value),
        }
    }
}

impl TryFrom<u8> for PerfMode {
    type Error = anyhow::Error;

    fn try_from(perf_mode: u8) -> Result<Self, Self::Error> {
        match perf_mode {
            0 => Ok(Self::Balanced),
            5 => Ok(Self::Silent),
            4 => Ok(Self::Custom),
            _ => bail!("Failed to convert {} to PerformanceMode", perf_mode),
        }
    }
}

impl TryFrom<u8> for FanMode {
    type Error = anyhow::Error;

    fn try_from(fan_mode: u8) -> Result<Self, Self::Error> {
        match fan_mode {
            0 => Ok(Self::Auto),
            1 => Ok(Self::Manual),
            _ => bail!("Failed to convert {} to FanMode", fan_mode),
        }
    }
}

impl TryFrom<u8> for CpuBoost {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::Medium),
            2 => Ok(Self::High),
            3 => Ok(Self::Boost),
            4 => Ok(Self::Overclock),
            _ => bail!("Failed to convert {} to CpuBoost", value),
        }
    }
}

impl TryFrom<u8> for LightsAlwaysOn {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LightsAlwaysOn::Disable),
            3 => Ok(LightsAlwaysOn::Enable),
            _ => bail!("Failed to convert {} to LightsAlwaysOn", value),
        }
    }
}

impl TryFrom<u8> for BatteryCare {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x50 => Ok(BatteryCare::Disable),
            0xd0 => Ok(BatteryCare::Enable),
            _ => bail!("Failed to convert {} to BatteryCare", value),
        }
    }
}

impl TryFrom<u8> for MaxFanSpeedMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x02 => Ok(MaxFanSpeedMode::Enable),
            0x00 => Ok(MaxFanSpeedMode::Disable),
            _ => bail!("Failed to convert {} to MaxFanSpeedMode", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_mode_try_from() {
        assert_eq!(PerfMode::try_from(0).unwrap(), PerfMode::Balanced);
        assert_eq!(PerfMode::try_from(5).unwrap(), PerfMode::Silent);
        assert_eq!(PerfMode::try_from(4).unwrap(), PerfMode::Custom);
        assert!(PerfMode::try_from(99).is_err());
    }

    #[test]
    fn test_fan_mode_try_from() {
        assert_eq!(FanMode::try_from(0).unwrap(), FanMode::Auto);
        assert_eq!(FanMode::try_from(1).unwrap(), FanMode::Manual);
        assert!(FanMode::try_from(2).is_err());
    }

    #[test]
    fn test_cpu_boost_try_from() {
        assert_eq!(CpuBoost::try_from(0).unwrap(), CpuBoost::Low);
        assert_eq!(CpuBoost::try_from(1).unwrap(), CpuBoost::Medium);
        assert_eq!(CpuBoost::try_from(2).unwrap(), CpuBoost::High);
        assert_eq!(CpuBoost::try_from(3).unwrap(), CpuBoost::Boost);
        assert_eq!(CpuBoost::try_from(4).unwrap(), CpuBoost::Overclock);
        assert!(CpuBoost::try_from(5).is_err());
    }

    #[test]
    fn test_gpu_boost_try_from() {
        assert_eq!(GpuBoost::try_from(0).unwrap(), GpuBoost::Low);
        assert_eq!(GpuBoost::try_from(1).unwrap(), GpuBoost::Medium);
        assert_eq!(GpuBoost::try_from(2).unwrap(), GpuBoost::High);
        assert!(GpuBoost::try_from(3).is_err());
    }

    #[test]
    fn test_lights_always_on_try_from() {
        assert_eq!(
            LightsAlwaysOn::try_from(0).unwrap(),
            LightsAlwaysOn::Disable
        );
        assert_eq!(LightsAlwaysOn::try_from(3).unwrap(), LightsAlwaysOn::Enable);
        assert!(LightsAlwaysOn::try_from(1).is_err());
    }

    #[test]
    fn test_battery_care_try_from() {
        assert_eq!(BatteryCare::try_from(0x50).unwrap(), BatteryCare::Disable);
        assert_eq!(BatteryCare::try_from(0xd0).unwrap(), BatteryCare::Enable);
        assert!(BatteryCare::try_from(0x00).is_err());
    }

    #[test]
    fn test_max_fan_speed_mode_try_from() {
        assert_eq!(
            MaxFanSpeedMode::try_from(0x00).unwrap(),
            MaxFanSpeedMode::Disable
        );
        assert_eq!(
            MaxFanSpeedMode::try_from(0x02).unwrap(),
            MaxFanSpeedMode::Enable
        );
        assert!(MaxFanSpeedMode::try_from(0x01).is_err());
    }
}
