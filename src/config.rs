//! 从配置文件加载，路径可由环境变量 CONFIG 或 PASTEBIN_CONFIG 覆盖。

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PastebinConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(default = "default_templates_dir")]
    pub templates_dir: String,

    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    #[serde(default = "default_static_dir")]
    pub static_dir: String,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_database_url() -> String {
    "sqlite://pastebin.db".to_string()
}
fn default_templates_dir() -> String {
    "templates".to_string()
}
fn default_data_dir() -> String {
    "data".to_string()
}
fn default_static_dir() -> String {
    "static".to_string()
}
fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8080
}

impl Default for PastebinConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
            templates_dir: default_templates_dir(),
            data_dir: default_data_dir(),
            static_dir: default_static_dir(),
            host: default_host(),
            port: default_port(),
        }
    }
}

/// 配置文件路径：环境变量 CONFIG 或 PASTEBIN_CONFIG，否则当前目录下的 pastebin.toml。
pub fn config_path() -> std::path::PathBuf {
    std::env::var("CONFIG")
        .or_else(|_| std::env::var("PASTEBIN_CONFIG"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("pastebin.toml"))
}

/// 从指定路径加载；文件不存在或解析失败时返回默认配置。
pub fn load(path: &Path) -> PastebinConfig {
    let s = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            log::debug!("config file {:?} not read: {}, using defaults", path, e);
            return PastebinConfig::default();
        }
    };
    match toml::from_str(&s) {
        Ok(c) => {
            log::info!("loaded config from {:?}", path);
            c
        }
        Err(e) => {
            log::warn!("config parse error in {:?}: {}, using defaults", path, e);
            PastebinConfig::default()
        }
    }
}
