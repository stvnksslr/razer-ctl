use const_format::{map_ascii_case, Case};

pub trait Feature {
    fn name(&self) -> &'static str;
}

macro_rules! feature_list {
    ($($type:ident,)*) => {
        $(
            #[derive(Default)]
            pub struct $type {}

            impl Feature for $type {
                fn name(&self) -> &'static str {
                    map_ascii_case!(Case::Kebab, stringify!($type))
                }
            }

            paste::paste! {
                #[doc = "Feature name constant for " $type]
                pub const [<$type:upper>]: &str = map_ascii_case!(Case::Kebab, stringify!($type));
            }
        )*

        pub const ALL_FEATURES: &[&'static str] = &[
            $(map_ascii_case!(Case::Kebab, stringify!($type)),)*
        ];

        #[macro_export]
        macro_rules! iter_features {
            ($apply:expr) => {
                {
                    let mut v = Vec::new();
                    $(
                        let entry = $type::default();
                        v.push($apply(entry.name(), entry));
                    )*
                    v
                }
            }
        }
    }
}

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

pub const fn validate_features(features: &[&str]) {
    const_for! { f in features => {
        assert!(contains(ALL_FEATURES, f), "Feature is not in supported list");
    }}
}

feature_list![
    BatteryCare,
    LidLogo,
    LightsAlwaysOn,
    KbdBacklight,
    Fan,
    Perf,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_names_are_kebab_case() {
        assert_eq!(BatteryCare::default().name(), "battery-care");
        assert_eq!(LidLogo::default().name(), "lid-logo");
        assert_eq!(LightsAlwaysOn::default().name(), "lights-always-on");
        assert_eq!(KbdBacklight::default().name(), "kbd-backlight");
        assert_eq!(Fan::default().name(), "fan");
        assert_eq!(Perf::default().name(), "perf");
    }

    #[test]
    fn test_feature_constants_match_names() {
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
