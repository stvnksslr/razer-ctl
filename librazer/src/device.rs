use crate::descriptor::{Descriptor, SUPPORTED};
use crate::error::{RazerError, Result};
use crate::packet::Packet;

use log::{debug, trace, warn};
#[cfg(target_os = "linux")]
use std::fs;
use std::{thread, time};

/// Result of enumerating connected Razer devices.
///
/// Contains the list of detected USB product IDs and the laptop model number prefix.
#[derive(Debug, Clone)]
pub struct EnumerationResult {
    /// List of USB product IDs for detected Razer devices.
    pub pids: Vec<u16>,
    /// Model number prefix (e.g., "RZ09-0483T").
    pub model: String,
}

/// Represents a connected Razer laptop device.
///
/// Wraps hidapi for USB HID communication. Use [`Device::detect`] for automatic
/// detection or [`Device::new`] with a specific [`Descriptor`] for manual setup.
pub struct Device {
    device: hidapi::HidDevice,
    /// Device descriptor containing model info and supported features.
    pub info: Descriptor,
}

// Read the model id and clip to conform with https://mysupport.razer.com/app/answers/detail/a_id/5481
#[cfg(target_os = "windows")]
fn read_device_model() -> Result<String> {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let bios = hklm
        .open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS")
        .map_err(|e| RazerError::ModelDetectionFailed(e.to_string()))?;
    let system_sku: String = bios
        .get_value("SystemSKU")
        .map_err(|e| RazerError::ModelDetectionFailed(e.to_string()))?;
    Ok(system_sku.chars().take(10).collect())
}

#[cfg(target_os = "linux")]
fn read_device_model() -> Result<String> {
    let sku = fs::read_to_string("/sys/devices/virtual/dmi/id/product_sku")
        .map(|s| s.trim().to_string())
        .map_err(|e| {
            RazerError::ModelDetectionFailed(format!("Failed to read product SKU: {}", e))
        })?;

    debug!("Linux product SKU: {}", sku);

    if sku.starts_with("RZ") {
        Ok(sku.chars().take(10).collect())
    } else {
        Err(RazerError::InvalidModel(sku))
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn read_device_model() -> Result<String> {
    debug!("Unsupported platform detected");
    Err(RazerError::UnsupportedPlatform)
}

impl Device {
    const RAZER_VID: u16 = 0x1532;

    /// Returns a reference to the device descriptor.
    pub fn info(&self) -> &Descriptor {
        &self.info
    }

    /// Creates a new Device with the specified descriptor.
    ///
    /// Opens the USB HID device matching the descriptor's PID.
    pub fn new(descriptor: Descriptor) -> Result<Device> {
        let api = hidapi::HidApi::new()?;

        // there are multiple devices with the same pid, pick first that support feature report
        let mut last_error: Option<String> = None;
        for info in api.device_list().filter(|info| {
            (info.vendor_id(), info.product_id()) == (Device::RAZER_VID, descriptor.pid)
        }) {
            let path = info.path();
            trace!("Trying to open device at path: {:?}", path);
            match api.open_path(path) {
                Ok(device) => {
                    trace!("Opened device, testing feature report...");
                    // Report ID (1 byte) + Packet (90 bytes) = 91 bytes total
                    match device.send_feature_report(&[0u8; 91]) {
                        Ok(_) => {
                            debug!(
                                "Connected to {} (PID: 0x{:04X})",
                                descriptor.name, descriptor.pid
                            );
                            return Ok(Device {
                                device,
                                info: descriptor.clone(),
                            });
                        }
                        Err(e) => {
                            debug!("Feature report failed on path {:?}: {}", path, e);
                            last_error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to open path {:?}: {}", path, e);
                    last_error = Some(e.to_string());
                }
            }
        }
        Err(RazerError::DeviceOpenFailed {
            name: descriptor.name.to_string(),
            reason: last_error.unwrap_or_else(|| "no matching device found".to_string()),
        })
    }

    /// Sends a USB HID feature report and returns the response.
    ///
    /// Handles the low-level protocol including timing delays and response validation.
    pub fn send(&self, report: Packet) -> Result<Packet> {
        // extra byte for report id
        let mut response_buf: Vec<u8> = vec![0x00; 1 + std::mem::size_of::<Packet>()];

        // Delay before sending to ensure device is ready for new command.
        // Per openrazer protocol, USB HID polling rate requires minimum inter-command spacing.
        thread::sleep(time::Duration::from_micros(1000));
        self.device.send_feature_report(
            [0_u8; 1] // report id
                .iter()
                .copied()
                .chain(Into::<Vec<u8>>::into(&report).into_iter())
                .collect::<Vec<_>>()
                .as_slice(),
        )?;

        // Delay before reading response to allow device to process command.
        // 2ms provides margin for device firmware to prepare response buffer.
        thread::sleep(time::Duration::from_micros(2000));
        let bytes_read = self.device.get_feature_report(&mut response_buf)?;
        if response_buf.len() != bytes_read {
            return Err(RazerError::InvalidDataSize {
                expected: response_buf.len(),
                actual: bytes_read,
            });
        }

        // skip report id byte
        let response = <&[u8] as TryInto<Packet>>::try_into(&response_buf[1..])?;
        response.ensure_matches_report(&report)
    }

    /// Enumerates connected Razer devices and detects the laptop model.
    ///
    /// Returns an [`EnumerationResult`] containing the list of PIDs found and
    /// the model number prefix (e.g., "RZ09-0483T").
    pub fn enumerate() -> Result<EnumerationResult> {
        let pids: Vec<_> = hidapi::HidApi::new()?
            .device_list()
            .filter(|info| info.vendor_id() == Device::RAZER_VID)
            .map(|info| info.product_id())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if pids.is_empty() {
            debug!("No Razer devices found in USB enumeration");
            return Err(RazerError::NoDevicesFound);
        }

        debug!("Found Razer devices with PIDs: {:?}", pids);

        match read_device_model() {
            Ok(model) => {
                debug!("Detected model: {}", model);
                if model.starts_with("RZ09-") {
                    Ok(EnumerationResult { pids, model })
                } else {
                    warn!("Model {} is not a Razer laptop (expected RZ09-*)", model);
                    Err(RazerError::InvalidModel(model))
                }
            }
            Err(e) => {
                warn!("Failed to detect model: {}", e);
                Err(e)
            }
        }
    }

    /// Auto-detects and connects to a supported Razer laptop.
    ///
    /// Combines [`enumerate`](Self::enumerate) with the [`SUPPORTED`] device list
    /// to find and open a compatible device.
    pub fn detect() -> Result<Device> {
        let enumeration = Device::enumerate()?;
        trace!("Looking for support for model: {}", enumeration.model);

        match SUPPORTED
            .iter()
            .find(|supported| enumeration.model.starts_with(supported.model_number_prefix))
        {
            Some(supported) => {
                debug!("Found supported device: {}", supported.name);
                Device::new(supported.clone())
            }
            None => {
                warn!(
                    "Model {} with PIDs {:0>4x?} is not supported",
                    enumeration.model, enumeration.pids
                );
                Err(RazerError::UnsupportedModel {
                    model: enumeration.model,
                    pids: enumeration.pids,
                })
            }
        }
    }
}
