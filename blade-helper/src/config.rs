use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "blade-helper";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub device: DeviceConfig,
    #[serde(default)]
    pub settings: SettingsConfig,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub cached_pid: Option<u16>,
    pub model: Option<String>,
    pub model_prefix: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub default_profile: Option<String>,
}

pub struct ConfigManager {
    config: Config,
    path: PathBuf,
}

impl ConfigManager {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let config: Config = confy::load(APP_NAME, None)?;
        Ok(Self { config, path })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn save(&self) -> Result<()> {
        confy::store(APP_NAME, None, &self.config)?;
        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn config_path() -> Result<PathBuf> {
        let path = confy::get_configuration_file_path(APP_NAME, None)?;
        Ok(path)
    }

    pub fn get_cached_pid(&self) -> Option<u16> {
        self.config.device.cached_pid
    }

    pub fn set_cached_device(&mut self, pid: u16, model: &str, model_prefix: &str) -> Result<()> {
        self.config.device.cached_pid = Some(pid);
        self.config.device.model = Some(model.to_string());
        self.config.device.model_prefix = Some(model_prefix.to_string());
        self.save()
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        self.config.device.cached_pid = None;
        self.config.device.model = None;
        self.config.device.model_prefix = None;
        self.save()
    }
}
