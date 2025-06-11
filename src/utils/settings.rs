// File: settings.rs
// Copyright (c) 2025 Anand Sureshkumar
//
// This source code is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License.
// See the LICENSE file or visit http://creativecommons.org/licenses/by-nc/4.0/ for details.
//
// Permission is granted to use, copy, and modify this code for personal, non-commercial, or educational purposes.
//
// Commercial use of this code, in whole or in part, is strictly prohibited without explicit written permission.
// For commercial licensing or other inquiries, contact: anandsuresh9988@gmail.com
//
// Disclaimer:
// This software interacts with external services (e.g., Trading 212 API) using user-provided credentials.
// The author is not responsible for any security vulnerabilities, data breaches, account lockouts,
// financial losses, or other issues arising from the use of this software.
//
// USE THIS SOFTWARE AT YOUR OWN RISK.

use std::{
    default,
    fs::File,
    io::{BufReader, BufWriter, Error as IoError},
    path::Path,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;

use super::currency::Currency;

const DEFAULT_PORTFOLIO_UPDATE_TIME_S: u64 = 60 * 60;
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] IoError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerdeError),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    Live,
    Demo,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Live => write!(f, "Live"),
            Mode::Demo => write!(f, "Demo"),
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Demo
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub currency: Currency,
    #[serde(default = "default_timeout")]
    pub portfolio_update_interval: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_PORTFOLIO_UPDATE_TIME_S)
}

impl Config {
    pub fn save_config(&self) -> Result<(), ConfigError> {
        let file = File::create(CONFIG_FILE)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load_config() -> Result<Self, ConfigError> {
        if !Path::new(CONFIG_FILE).exists() {
            let config = Config::default();
            let _ = config.save_config();
        }
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
