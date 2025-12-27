use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No Razer device found. Make sure your device is connected and supported.")]
    DeviceNotFound,

    #[error("Permission denied accessing USB device. On Linux, install udev rules: see README for details.")]
    PermissionDenied,

    #[error("Feature '{0}' is not supported on this device")]
    FeatureNotSupported(String),

    #[error("Configuration error: {0}")]
    Config(#[from] confy::ConfyError),

    #[error("Device error: {0}")]
    Device(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
