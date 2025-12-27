//! Feature constants for device capability validation.
//!
//! Features are validated at compile time to ensure only valid
//! feature strings are used in device descriptors.

/// Feature name for battery care mode (80% charge limit)
pub const BATTERYCARE: &str = "battery-care";
/// Feature name for lid logo control
pub const LIDLOGO: &str = "lid-logo";
/// Feature name for lights-always-on setting
pub const LIGHTSALWAYSON: &str = "lights-always-on";
/// Feature name for keyboard backlight control
pub const KBDBACKLIGHT: &str = "kbd-backlight";
/// Feature name for fan control
pub const FAN: &str = "fan";
/// Feature name for performance mode control
pub const PERF: &str = "perf";

/// All valid feature names for compile-time validation
pub const ALL_FEATURES: &[&str] = &[
    BATTERYCARE,
    LIDLOGO,
    LIGHTSALWAYSON,
    KBDBACKLIGHT,
    FAN,
    PERF,
];

/// Helper macro for const iteration over slices
#[macro_export]
macro_rules! const_for {
    ($var:ident in $iter:expr => $block:block) => {
        let mut iter = $iter;
        while let [$var, tail @ ..] = iter {
            iter = tail;
            $block
        }
    };
}

const fn contains(array: &[&str], value: &str) -> bool {
    const_for! { it in array => {
        if const_str::equal!(*it, value) {
            return true;
        }
    }}
    false
}

/// Validates that all feature strings are in the supported list.
/// Called at compile time from descriptor.rs.
pub const fn validate_features(features: &[&str]) {
    const_for! { f in features => {
        assert!(contains(ALL_FEATURES, f), "Feature is not in supported list");
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_constants() {
        assert_eq!(BATTERYCARE, "battery-care");
        assert_eq!(LIDLOGO, "lid-logo");
        assert_eq!(LIGHTSALWAYSON, "lights-always-on");
        assert_eq!(KBDBACKLIGHT, "kbd-backlight");
        assert_eq!(FAN, "fan");
        assert_eq!(PERF, "perf");
    }

    #[test]
    fn test_all_features_contains_all() {
        assert!(ALL_FEATURES.contains(&"battery-care"));
        assert!(ALL_FEATURES.contains(&"lid-logo"));
        assert!(ALL_FEATURES.contains(&"lights-always-on"));
        assert!(ALL_FEATURES.contains(&"kbd-backlight"));
        assert!(ALL_FEATURES.contains(&"fan"));
        assert!(ALL_FEATURES.contains(&"perf"));
        assert_eq!(ALL_FEATURES.len(), 6);
    }

    #[test]
    fn test_validate_features_accepts_valid() {
        // Should not panic
        validate_features(&["battery-care", "fan", "perf"]);
    }
}
