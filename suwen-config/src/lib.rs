use anyhow::{Context, Result};
use dirs::config_dir;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::{fs, sync::LazyLock};

pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| Config::init().expect("Failed to initialize config"));

fn random_string(length: usize) -> String {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| chars.chars().choose(&mut rng).unwrap())
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub jwt_secret: String,
    pub openai_api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jwt_secret: random_string(32),
            openai_api_key: String::new(),
        }
    }
}

impl Config {
    pub fn init() -> Result<Config> {
        let config_path = config_dir()
            .map(|path| path.join("suwen").join("suwen.json"))
            .unwrap_or_else(|| "suwen.json".into());
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                Ok(serde_json::from_str(&content).context("Failed to parse config file")?)
            }
            Err(_) => {
                let config = Config::default();
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent).context("Failed to create config directory")?;
                }
                fs::write(config_path, serde_json::to_string(&config)?)
                    .context("Failed to write default config file")?;
                Ok(config)
            }
        }
    }
}
