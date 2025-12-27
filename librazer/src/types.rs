use crate::error::RazerError;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

/// Generates TryFrom<u8> implementation for enums with explicit discriminants.
macro_rules! impl_try_from_u8 {
    ($enum_type:ident { $($value:expr => $variant:ident),+ $(,)? }) => {
        impl TryFrom<u8> for $enum_type {
            type Error = RazerError;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(RazerError::InvalidValue {
                        value,
                        type_name: stringify!($enum_type),
                    }),
                }
            }
        }
    };
}

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

impl FanZone {
    /// Both fan zones for operations that affect all fans
    pub const ALL: [FanZone; 2] = [FanZone::Zone1, FanZone::Zone2];
}

/// Thermal zones for performance mode operations
#[derive(Clone, Copy)]
pub enum ThermalZone {
    Zone1 = 0x01,
    Zone2 = 0x02,
}

impl ThermalZone {
    /// Both thermal zones for operations that affect the entire thermal system
    pub const ALL: [ThermalZone; 2] = [ThermalZone::Zone1, ThermalZone::Zone2];
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, ValueEnum)]
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

#[derive(
    Clone, Copy, Debug, PartialEq, Serialize, Deserialize, EnumIter, EnumString, ValueEnum,
)]
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

impl_try_from_u8!(GpuBoost { 0 => Low, 1 => Medium, 2 => High });
impl_try_from_u8!(PerfMode { 0 => Balanced, 5 => Silent, 4 => Custom });
impl_try_from_u8!(FanMode { 0 => Auto, 1 => Manual });
impl_try_from_u8!(CpuBoost { 0 => Low, 1 => Medium, 2 => High, 3 => Boost, 4 => Overclock });
impl_try_from_u8!(LightsAlwaysOn { 0 => Disable, 3 => Enable });
impl_try_from_u8!(BatteryCare { 0x50 => Disable, 0xd0 => Enable });
impl_try_from_u8!(MaxFanSpeedMode { 0x00 => Disable, 0x02 => Enable });

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
