use thiserror::Error;

/// Errors that can occur when communicating with Razer devices.
#[derive(Error, Debug)]
pub enum RazerError {
    /// No Razer devices were found on the USB bus.
    #[error("No Razer devices found")]
    NoDevicesFound,

    /// The device does not support this command.
    #[error("Command not supported by device")]
    CommandNotSupported,

    /// The device is busy processing a previous command.
    #[error("Device busy, try again")]
    DeviceBusy,

    /// The command failed to execute.
    #[error("Command failed")]
    CommandFailed,

    /// The command timed out waiting for a response.
    #[error("Command timed out")]
    CommandTimeout,

    /// The device returned an unknown status code.
    #[error("Command failed with unknown status: 0x{0:02X}")]
    UnknownStatus(u8),

    /// The response packet does not match the sent command.
    #[error("Response does not match the report")]
    ResponseMismatch,

    /// Failed to read the device model from the system.
    #[error("Failed to detect model: {0}")]
    ModelDetectionFailed(String),

    /// The detected model is not a Razer laptop.
    #[error("Detected model but it's not a Razer laptop: {0}")]
    InvalidModel(String),

    /// The device model is not in the supported device list.
    #[error("Model {model} with PIDs {pids:0>4x?} is not supported")]
    UnsupportedModel { model: String, pids: Vec<u16> },

    /// Automatic model detection is not available on this platform.
    #[error("Automatic model detection is not implemented for this platform")]
    UnsupportedPlatform,

    /// Failed to open the USB HID device.
    #[error("Failed to open device {name:?}: {reason}")]
    DeviceOpenFailed { name: String, reason: String },

    /// Invalid value when converting from raw bytes.
    #[error("Failed to convert {value} to {type_name}")]
    InvalidValue { value: u8, type_name: &'static str },

    /// Invalid data size in packet or response.
    #[error("Invalid data size: expected {expected}, got {actual}")]
    InvalidDataSize { expected: usize, actual: usize },

    /// USB HID communication error.
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),

    /// Precondition not met for the command.
    #[error("{0}")]
    PreconditionFailed(String),

    /// Generic error for other cases.
    #[error("{0}")]
    Other(String),
}

/// Result type alias using [`RazerError`].
pub type Result<T> = std::result::Result<T, RazerError>;
