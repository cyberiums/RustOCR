use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub default: Option<DefaultConfig>,
    pub server: Option<ServerConfig>,
    pub batch: Option<BatchConfig>,
    pub profiles: Option<HashMap<String, ProfileConfig>>,
}

/// Default OCR settings
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultConfig {
    pub languages: Option<Vec<String>>,
    pub gpu: Option<bool>,
    pub output: Option<String>,
    pub detail: Option<i32>,
}

/// Server configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub enabled: Option<bool>,
    pub port: Option<u16>,
    pub host: Option<String>,
    pub auto_start: Option<bool>,
}

/// Batch processing configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatchConfig {
    pub output_dir: Option<String>,
    pub continue_on_error: Option<bool>,
}

/// Named profile configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileConfig {
    pub languages: Option<Vec<String>>,
    pub gpu: Option<bool>,
    pub output: Option<String>,
    pub detail: Option<i32>,
}

impl Config {
    /// Load configuration from hierarchy: project > user > system
    pub fn load() -> Result<Self> {
        let mut configs = Vec::new();

        // 1. System config
        if let Some(system) = Self::load_from(&PathBuf::from("/etc/rustocr/config.toml"))? {
            configs.push(system);
        }

        // 2. User config
        if let Some(config_dir) = dirs::config_dir() {
            let user_config_path = config_dir.join("rustocr").join("config.toml");
            if let Some(user) = Self::load_from(&user_config_path)? {
                configs.push(user);
            }
        }

        // 3. Project config
        if let Some(project) = Self::load_from(&PathBuf::from("./rustocr.toml"))? {
            configs.push(project);
        }

        // Merge configs (later configs override earlier ones)
        Ok(Self::merge(configs))
    }

    /// Load configuration from a specific file
    pub fn load_from(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(Some(config))
    }

    /// Merge multiple configs (later configs take precedence)
    fn merge(configs: Vec<Self>) -> Self {
        let mut result = Self::default();

        for config in configs {
            if let Some(default) = config.default {
                result.default = Some(Self::merge_default(
                    result.default.take(),
                    default,
                ));
            }

            if let Some(server) = config.server {
                result.server = Some(Self::merge_server(
                    result.server.take(),
                    server,
                ));
            }

            if let Some(batch) = config.batch {
                result.batch = Some(Self::merge_batch(
                    result.batch.take(),
                    batch,
                ));
            }

            if let Some(profiles) = config.profiles {
                let mut merged_profiles = result.profiles.take().unwrap_or_default();
                merged_profiles.extend(profiles);
                result.profiles = Some(merged_profiles);
            }
        }

        result
    }

    fn merge_default(base: Option<DefaultConfig>, override_cfg: DefaultConfig) -> DefaultConfig {
        let mut result = base.unwrap_or(DefaultConfig {
            languages: None,
            gpu: None,
            output: None,
            detail: None,
        });

        if let Some(langs) = override_cfg.languages {
            result.languages = Some(langs);
        }
        if let Some(gpu) = override_cfg.gpu {
            result.gpu = Some(gpu);
        }
        if let Some(output) = override_cfg.output {
            result.output = Some(output);
        }
        if let Some(detail) = override_cfg.detail {
            result.detail = Some(detail);
        }

        result
    }

    fn merge_server(base: Option<ServerConfig>, override_cfg: ServerConfig) -> ServerConfig {
        let mut result = base.unwrap_or(ServerConfig {
            enabled: None,
            port: None,
            host: None,
            auto_start: None,
        });

        if let Some(enabled) = override_cfg.enabled {
            result.enabled = Some(enabled);
        }
        if let Some(port) = override_cfg.port {
            result.port = Some(port);
        }
        if let Some(host) = override_cfg.host {
            result.host = Some(host);
        }
        if let Some(auto_start) = override_cfg.auto_start {
            result.auto_start = Some(auto_start);
        }

        result
    }

    fn merge_batch(base: Option<BatchConfig>, override_cfg: BatchConfig) -> BatchConfig {
        let mut result = base.unwrap_or(BatchConfig {
            output_dir: None,
            continue_on_error: None,
        });

        if let Some(output_dir) = override_cfg.output_dir {
            result.output_dir = Some(output_dir);
        }
        if let Some(continue_on_error) = override_cfg.continue_on_error {
            result.continue_on_error = Some(continue_on_error);
        }

        result
    }

    /// Create default config file template
    pub fn create_default() -> String {
        r#"# RustOCR Configuration File
# Save as: ~/.config/rustocr/config.toml or ./rustocr.toml

[default]
languages = ["en"]
gpu = true
output = "json"
detail = 1

[server]
enabled = false
port = 8000
host = "127.0.0.1"
auto_start = false

[batch]
output_dir = "./results"
continue_on_error = true

# Example profiles
[profiles.chinese]
languages = ["ch_sim", "en"]
gpu = true
detail = 1

[profiles.multilang]
languages = ["en", "ch_sim", "ja", "ko"]
gpu = true
output = "detailed"
detail = 1

[profiles.fast]
languages = ["en"]
gpu = true
detail = 0
"#.to_string()
    }
}
