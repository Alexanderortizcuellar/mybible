use crate::error::{MyBibleError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Persistent CLI configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub data_dir: Option<String>,
    pub default_bible: Option<String>,
    pub default_commentary: Option<String>,
    pub default_dictionary: Option<String>,
    pub default_subheadings: Option<String>,
}

impl CliConfig {
    /// Loads the configuration from the user's config directory (~/.mybible/config.json).
    pub fn load() -> Self {
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(cfg) = serde_json::from_str(&content) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }

    /// Saves the current configuration to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_file_path().ok_or_else(|| {
            MyBibleError::InvalidFormat("Cannot determine config directory".to_string())
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| MyBibleError::InvalidFormat(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Returns the path to `~/.mybible/config.json`.
    pub fn config_file_path() -> Option<PathBuf> {
        let base = dirs::home_dir()?;
        Some(base.join(".mybible").join("config.json"))
    }

    /// Resolves the active modules data directory.
    ///
    /// Precedence:
    /// 1. Explicit override passed via CLI (`--dir <PATH>`)
    /// 2. Environment variable `MYBIBLE_DATA_DIR`
    /// 3. Saved `data_dir` in `~/.mybible/config.json`
    /// 4. Current working directory `./modules` (if it exists)
    /// 5. Standard default: `~/.mybible/modules`
    pub fn resolve_data_dir(cli_dir_override: Option<&Path>) -> Result<PathBuf> {
        // 1. CLI Override
        if let Some(dir) = cli_dir_override {
            fs::create_dir_all(dir)?;
            return Ok(dir.to_path_buf());
        }

        // 2. Environment Variable
        if let Ok(env_val) = std::env::var("MYBIBLE_DATA_DIR") {
            let p = PathBuf::from(env_val);
            fs::create_dir_all(&p)?;
            return Ok(p);
        }

        // 3. Saved Config
        let cfg = Self::load();
        if let Some(saved) = &cfg.data_dir {
            let p = PathBuf::from(saved);
            fs::create_dir_all(&p)?;
            return Ok(p);
        }

        // 4. Current working directory `./modules`
        let local_modules = PathBuf::from("modules");
        if local_modules.exists() && local_modules.is_dir() {
            return Ok(local_modules);
        }

        // 5. Default: ~/.mybible/modules
        let default_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mybible")
            .join("modules");

        fs::create_dir_all(&default_dir)?;
        Ok(default_dir)
    }
}
