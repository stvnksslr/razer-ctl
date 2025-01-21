use crate::descriptor::{Descriptor, SUPPORTED};
use crate::packet::Packet;

use anyhow::{anyhow, Context, Result};
use std::{fs, thread, time};

pub struct Device {
    device: hidapi::HidDevice,
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

    // println!("[DEBUG] Linux product SKU: {}", sku);

    if sku.starts_with("RZ") {
        Ok(sku.chars().take(10).collect())
    } else {
        anyhow::bail!("Invalid Razer laptop SKU: {}", sku)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn read_device_model() -> Result<String> {
    // println!("[DEBUG] Unsupported platform detected");
    anyhow::bail!("Automatic model detection is not implemented for this platform")
}

impl Device {
    const RAZER_VID: u16 = 0x1532;

    pub fn info(&self) -> &Descriptor {
        &self.info
    }

    pub fn new(descriptor: Descriptor) -> Result<Device> {
        let api = hidapi::HidApi::new().context("Failed to create hid api")?;

        // there are multiple devices with the same pid, pick first that support feature report
        for info in api.device_list().filter(|info| {
            (info.vendor_id(), info.product_id()) == (Device::RAZER_VID, descriptor.pid)
        }) {
            let path = info.path();
            let device = api.open_path(path)?;
            if device.send_feature_report(&[0, 0]).is_ok() {
                return Ok(Device {
                    device,
                    info: descriptor.clone(),
                });
            }
        }
        anyhow::bail!("Failed to open device {:?}", descriptor)
    }

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

    pub fn enumerate() -> Result<(Vec<u16>, String)> {
        let razer_pid_list: Vec<_> = hidapi::HidApi::new()?
            .device_list()
            .filter(|info| info.vendor_id() == Device::RAZER_VID)
            .map(|info| info.product_id())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if razer_pid_list.is_empty() {
            // println!("[DEBUG] No Razer devices found in USB enumeration");
            anyhow::bail!("No Razer devices found")
        }

        // println!(
        //     "[DEBUG] Found Razer devices with PIDs: {:?}",
        //     razer_pid_list
        // );

        match read_device_model() {
            Ok(model) => {
                // println!("[DEBUG] Detected model number: {}", model);
                if model.starts_with("RZ09-") {
                    Ok((razer_pid_list, model))
                } else {
                    anyhow::bail!("Detected model but it's not a Razer laptop: {}", model)
                }
            }
            Err(e) => {
                // println!("[DEBUG] Failed to detect model: {}", e);
                anyhow::bail!("Failed to detect model: {}", e)
            }
        }
    }

    pub fn detect() -> Result<Device> {
        let (pid_list, model_number_prefix) = Device::enumerate()?;
        // println!(
        //     "[DEBUG] Looking for support for model: {}",
        //     model_number_prefix
        // );

        match SUPPORTED
            .iter()
            .find(|supported| model_number_prefix.starts_with(supported.model_number_prefix))
        {
            Some(supported) => {
                // println!("[DEBUG] Found supported device: {:?}", supported);
                Device::new(supported.clone())
            }
            None => {
                // println!("[DEBUG] Model not supported");
                anyhow::bail!(
                    "Model {} with PIDs {:0>4x?} is not supported",
                    model_number_prefix,
                    pid_list
                )
            }
        }
    }
}
