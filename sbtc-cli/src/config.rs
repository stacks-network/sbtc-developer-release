use std::{
    fs::{create_dir_all, read_to_string},
    io::Write,
};

use bdk::bitcoin::Network;
use serde::Deserialize;
use url::Url;

pub const GENERATED_CONFIG: &str = include_str!("../generated_config.toml");

pub fn generate_config() -> anyhow::Result<()> {
    let home_path = dirs::home_dir().unwrap();
    let config_path = home_path.join(".config/sbtc/config.toml");

    create_dir_all(config_path.parent().unwrap())?;

    if config_path.exists() {
        println!(
            "Config file already exists at {}",
            config_path.to_str().unwrap()
        );
        return Ok(());
    }

    let mut config_file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&config_path)
        .unwrap();

    config_file.write_all(GENERATED_CONFIG.as_bytes())?;
    config_file.flush()?;

    println!(
        "Config file created at {}, make sure to update it before using",
        config_path.to_str().unwrap()
    );

    Ok(())
}

pub fn read_config() -> anyhow::Result<Config> {
    let home_path = dirs::home_dir().unwrap();
    let config_path = home_path.join(".config/sbtc/config.toml");

    let config_file = read_to_string(config_path)?;

    Ok(toml::from_str(&config_file)?)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bitcoin_node_url: Url,
    pub bitcoin_network: Network,
}
