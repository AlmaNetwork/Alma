use serde::Deserialize;
use std::fs;
use anyhow::{Result, Context};

#[derive(Deserialize, Default)]
pub struct NetworkConfig {
    #[serde(default = "default_port")]
    pub port: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_remote_address")]
    pub remote_address: String,
}

fn default_port() -> String {
    "8080".to_string()
}

fn default_mode() -> String {
    "answer".to_string()
}

fn default_remote_address() -> String {
    "127.0.0.1:8081".to_string()
}

#[derive(Deserialize)]
struct Config {
    network: NetworkConfig,
}

pub fn load_config(path: &str) -> Result<NetworkConfig> {
    match fs::read_to_string(path) {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str)
                .with_context(|| format!("Failed to parse config file: {}", path))?;
            Ok(config.network)
        }
        Err(_) => {
            println!("Config file not found. Using default values and command line arguments.");
            Ok(NetworkConfig::default())
        }
    }
}