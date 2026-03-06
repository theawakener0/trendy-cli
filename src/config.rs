use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_path = config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let config_path = config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

fn config_path() -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;
    Ok(config_dir.join("trendy-cli").join("config.json"))
}

pub fn get_api_key(cli_key: Option<String>) -> Option<String> {
    if let Some(key) = cli_key {
        let config = Config {
            api_key: Some(key.clone()),
        };
        let _ = config.save();
        return Some(key);
    }

    if let Ok(config) = Config::load() {
        if let Some(key) = config.api_key {
            return Some(key);
        }
    }

    std::env::var("HACKCLUB_API_KEY").ok()
}
