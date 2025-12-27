use crate::descriptor::{Descriptor, SUPPORTED};
use crate::packet::Packet;

use anyhow::{anyhow, Context, Result};
use log::{debug, error};
#[cfg(target_os = "linux")]
use std::fs;
use std::{thread, time};

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
    let bios = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS")?;
    let system_sku: String = bios.get_value("SystemSKU")?;
    Ok(system_sku.chars().take(10).collect())
}

#[cfg(target_os = "linux")]
fn read_device_model() -> Result<String> {
    let sku = fs::read_to_string("/sys/devices/virtual/dmi/id/product_sku")
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow::anyhow!("Failed to read product SKU: {}", e))?;

    debug!("Linux product SKU: {}", sku);

    if sku.starts_with("RZ") {
        Ok(sku.chars().take(10).collect())
    } else {
        anyhow::bail!("Invalid Razer laptop SKU: {}", sku)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn read_device_model() -> Result<String> {
    debug!("Unsupported platform detected");
    anyhow::bail!("Automatic model detection is not implemented for this platform")
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
        let api = hidapi::HidApi::new().context("Failed to create hid api")?;

        // there are multiple devices with the same pid, pick first that support feature report
        let mut last_error: Option<String> = None;
        for info in api.device_list().filter(|info| {
            (info.vendor_id(), info.product_id()) == (Device::RAZER_VID, descriptor.pid)
        }) {
            let path = info.path();
            debug!("Trying to open device at path: {:?}", path);
            match api.open_path(path) {
                Ok(device) => {
                    debug!("Opened device, testing feature report...");
                    // Report ID (1 byte) + Packet (90 bytes) = 91 bytes total
                    match device.send_feature_report(&[0u8; 91]) {
                        Ok(_) => {
                            debug!("Feature report succeeded");
                            return Ok(Device {
                                device,
                                info: descriptor.clone(),
                            });
                        }
                        Err(e) => {
                            debug!("Feature report failed: {}", e);
                            last_error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to open path: {}", e);
                    last_error = Some(e.to_string());
                }
            }
        }
        anyhow::bail!(
            "Failed to open device {:?}: {}",
            descriptor,
            last_error.unwrap_or_else(|| "no matching device found".to_string())
        )
    }

    /// Sends a USB HID feature report and returns the response.
    ///
    /// Handles the low-level protocol including timing delays and response validation.
    pub fn send(&self, report: Packet) -> Result<Packet> {
        // extra byte for report id
        let mut response_buf: Vec<u8> = vec![0x00; 1 + std::mem::size_of::<Packet>()];

        thread::sleep(time::Duration::from_micros(1000));
        self.device
            .send_feature_report(
                [0_u8; 1] // report id
                    .iter()
                    .copied()
                    .chain(Into::<Vec<u8>>::into(&report).into_iter())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .context("Failed to send feature report")?;

        thread::sleep(time::Duration::from_micros(2000));
        if response_buf.len() != self.device.get_feature_report(&mut response_buf)? {
            return Err(anyhow!("Response size != {}", response_buf.len()));
        }

        // skip report id byte
        let response = <&[u8] as TryInto<Packet>>::try_into(&response_buf[1..])?;
        response.ensure_matches_report(&report)
    }

    /// Enumerates connected Razer devices and detects the laptop model.
    ///
    /// Returns a list of PIDs found and the model number prefix (e.g., "RZ09-0483T").
    pub fn enumerate() -> Result<(Vec<u16>, String)> {
        let razer_pid_list: Vec<_> = hidapi::HidApi::new()?
            .device_list()
            .filter(|info| info.vendor_id() == Device::RAZER_VID)
            .map(|info| info.product_id())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if razer_pid_list.is_empty() {
            debug!("No Razer devices found in USB enumeration");
            anyhow::bail!("No Razer devices found")
        }

        debug!("Found Razer devices with PIDs: {:?}", razer_pid_list);

        match read_device_model() {
            Ok(model) => {
                debug!("Detected model number: {}", model);
                if model.starts_with("RZ09-") {
                    Ok((razer_pid_list, model))
                } else {
                    error!("Detected model but it's not a Razer laptop: {}", model);
                    anyhow::bail!("Detected model but it's not a Razer laptop: {}", model)
                }
            }
            Err(e) => {
                error!("Failed to detect model: {}", e);
                anyhow::bail!("Failed to detect model: {}", e)
            }
        }
    }

    /// Auto-detects and connects to a supported Razer laptop.
    ///
    /// Combines [`enumerate`](Self::enumerate) with the [`SUPPORTED`] device list
    /// to find and open a compatible device.
    pub fn detect() -> Result<Device> {
        let (pid_list, model_number_prefix) = Device::enumerate()?;
        debug!("Looking for support for model: {}", model_number_prefix);

        match SUPPORTED
            .iter()
            .find(|supported| model_number_prefix.starts_with(supported.model_number_prefix))
        {
            Some(supported) => {
                debug!("Found supported device: {:?}", supported);
                Device::new(supported.clone())
            }
            None => {
                debug!("Model not supported");
                anyhow::bail!(
                    "Model {} with PIDs {:0>4x?} is not supported",
                    model_number_prefix,
                    pid_list
                )
            }
        }
    }
}
