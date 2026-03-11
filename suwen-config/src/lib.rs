use std::fmt::{Display, Formatter};
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
    pub openai_base_url: Option<String>,
    pub openai_api_key: String,
    pub openai_model: String,
    pub host_url: String,
    pub r2: R2Config,
    #[serde(default)]
    pub markdown_path: Option<String>,
    #[serde(default)]
    pub source_lang: Lang,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct R2Config {
    pub bucket_name: String,
    pub account_id: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub prefix: String,
    #[serde(default = "default_s3_domain")]
    pub s3_domain: String,
}

fn default_s3_domain() -> String {
    "https://obj.amto.cc".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Lang {
    #[default]
    ZhCN,
    EnUS,
    JaJP,
    KoKR,
}

impl Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::ZhCN => write!(f, "zh-CN"),
            Lang::EnUS => write!(f, "en-US"),
            Lang::JaJP => write!(f, "ja-JP"),
            Lang::KoKR => write!(f, "ko-KR"),
        }
    }
}

impl TryFrom<&str> for Lang {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "zh-CN" => Ok(Lang::ZhCN),
            "en-US" => Ok(Lang::EnUS),
            "ja-JP" => Ok(Lang::JaJP),
            "ko-KR" => Ok(Lang::KoKR),
            _ => Err("Unsupported language code"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jwt_secret: random_string(32),
            openai_api_key: String::new(),
            openai_base_url: None,
            openai_model: String::new(),
            host_url: String::new(),
            r2: R2Config::default(),
            markdown_path: None,
            source_lang: Default::default(),
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
