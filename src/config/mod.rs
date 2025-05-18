use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookConfig {
    pub system: String,
    pub usb: String,
    pub idle: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub device_name: String,
    pub webhooks: WebhookConfig,
    pub ping_interval: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        
        let mut file = File::open(&config_path)
            .with_context(|| format!("Failed to open config file at {:?}", config_path))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read config file contents")?;
        
        let config: Config = serde_json::from_str(&contents)
            .context("Failed to parse config.json")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config to JSON")?;
        
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file to {:?}", config_path))?;
        
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|home| home.join("Library/Application Support/RAA/config.json"))
            .context("Failed to determine home directory on macOS")
    }
    
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .map(|path| path.join("RAA").join("config.json"))
            .context("Failed to determine config directory on Windows")
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other platforms (Linux, etc.)
        dirs::config_dir()
            .map(|path| path.join("raa").join("config.json"))
            .context("Failed to determine config directory")
    }
}

pub fn create_default_config() -> Result<Config> {
    let hostname = gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "Unknown Device".to_string());
    
    let config = Config {
        device_name: format!("{}", hostname),
        webhooks: WebhookConfig {
            system: "https://discord.com/api/webhooks/system".to_string(),
            usb: "https://discord.com/api/webhooks/usb".to_string(),
            idle: "https://discord.com/api/webhooks/idle".to_string(),
        },
        ping_interval: 15,
    };
    
    config.save()?;
    Ok(config)
}

pub fn ensure_config_exists() -> Result<Config> {
    let config_path = get_config_path()?;
    
    if config_path.exists() {
        Config::load()
    } else {
        log::info!("Config file not found, creating default at {:?}", config_path);
        create_default_config()
    }
}
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        log::info!("Config file not found, creating default at {:?}", config_path);
        return create_default_config();
    }
    
    match Config::load() {
        Ok(config) => Ok(config),
        Err(err) => {
            log::error!("Failed to load config: {}", err);
            log::info!("Creating default config");
            create_default_config()
        }
    }
}
