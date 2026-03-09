use std::fs;
use std::sync::LazyLock;

use anyhow::{Context, Result};
use dirs::config_dir;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::init().expect("Failed to initialize config"));

fn random_string(length: usize) -> String {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length).map(|_| chars.chars().choose(&mut rng).unwrap()).collect()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub jwt_secret: String,
    pub openai_api_key: String,
    #[serde(default)]
    pub host_url: Option<String>,
    #[serde(default)]
    pub r2: Option<R2Config>,
    #[serde(default = "default_object_storage_domain")]
    pub object_storage_domain: String,
    #[serde(default)]
    pub markdown_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct R2Config {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
}

fn default_object_storage_domain() -> String {
    "https://obj.amto.cc".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jwt_secret: random_string(32),
            openai_api_key: String::new(),
            host_url: None,
            r2: None,
            object_storage_domain: default_object_storage_domain(),
            markdown_path: None,
        }
    }
}

impl Config {
    pub fn init() -> Result<Config> {
        let config_path = config_dir()
            .map(|path| path.join("suwen").join("suwen.json"))
            .unwrap_or_else(|| "suwen.json".into());
        match std::fs::read_to_string(&config_path) {
            Ok(content) => Ok(serde_json::from_str(&content).context("Failed to parse config file")?),
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
