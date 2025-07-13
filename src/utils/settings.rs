// File: settings.rs
// Copyright (c) 2025 Anand Sureshkumar
// This file is part of T212 Portfolio Analytics.
// Licensed for personal and educational use only. Commercial use prohibited.
// See the LICENSE file for details.
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
    fs::File,
    io::{BufReader, BufWriter, Error as IoError},
    path::Path,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;

use super::currency::Currency;

/// Default portfolio update interval in seconds (1 hour)
const DEFAULT_PORTFOLIO_UPDATE_TIME_S: u64 = 60 * 60;
/// Configuration file name
const CONFIG_FILE: &str = "config.json";

/// Custom error types for configuration operations
///
/// This enum defines the possible errors that can occur when
/// loading or saving configuration files.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Error when file I/O operations fail
    #[error("IO error: {0}")]
    Io(#[from] IoError),
    /// Error when JSON serialization/deserialization fails
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerdeError),
}

/// Application running mode
///
/// This enum defines whether the application is running in live trading mode
/// or demo mode. Demo mode is added to help someone to test the application
/// with some demo data. This allow people to test the application without
/// having a T212 account.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    /// Live  mode
    Live,
    /// Demo mode
    Demo,
}

impl std::fmt::Display for Mode {
    /// Converts the Mode enum to its string representation
    ///
    /// # Returns
    /// - `&'static str` containing the mode name
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Live => write!(f, "Live"),
            Mode::Demo => write!(f, "Demo"),
        }
    }
}

impl Default for Mode {
    /// Returns the default mode
    fn default() -> Self {
        Mode::Demo
    }
}

/// Config structure for the application
///
/// This struct holds all the configuration settings that control
/// the behavior of the portfolio analytics application.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    ///  Application mode (Live or Demo)
    pub mode: Mode,
    /// API key for Trading 212 authentication (optional for demo mode)
    pub api_key: Option<String>,
    /// Default currency for portfolio calculations
    pub currency: Currency,
    /// Interval between portfolio updates in seconds
    pub portfolio_update_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            api_key: None,
            currency: Currency::default(),
            portfolio_update_interval: default_timeout(),
        }
    }
}

/// Returns the default portfolio update interval
///
/// This function provides the default duration for portfolio updates.
/// It's used as a default value in the Config struct.
///
/// # Returns
/// - `Duration` representing update interval in seconds.
fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_PORTFOLIO_UPDATE_TIME_S)
}

impl Config {
    /// Saves the current configuration to the config file
    ///
    /// This method serializes the Config struct to JSON and writes it
    /// to the configuration file. If the file doesn't exist, it will
    /// be created.
    ///
    /// # Returns
    /// - `Ok(())` on successful save
    /// - `Err(ConfigError::Io)` if file creation/writing fails
    /// - `Err(ConfigError::Serialization)` if JSON serialization fails
    ///
    /// # Example
    /// ```ignore
    /// let config = Config::default();
    /// config.save_config()?;
    /// ```
    pub fn save_config(&self) -> Result<(), ConfigError> {
        let file = File::create(CONFIG_FILE)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Loads configuration from the config file
    ///
    /// This method reads the configuration file and deserializes it
    /// into a Config struct. If the file doesn't exist, a default
    /// configuration is created and saved.
    ///
    /// # Returns
    /// - `Ok(Config)` containing the loaded configuration
    /// - `Err(ConfigError::Io)` if file reading fails
    /// - `Err(ConfigError::Serialization)` if JSON deserialization fails
    ///
    /// # Example
    /// ```ignore
    /// let config = Config::load_config()?;
    /// println!("Mode: {}", config.mode);
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test that Config has correct default values
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.mode, Mode::Demo);
        assert_eq!(config.api_key, None);
        assert_eq!(config.currency, Currency::GBP);
        assert_eq!(
            config.portfolio_update_interval,
            Duration::from_secs(DEFAULT_PORTFOLIO_UPDATE_TIME_S)
        );
    }

    /// Test Config serialization and deserialization
    #[test]
    fn test_config_serialization() {
        let config = Config {
            mode: Mode::Live,
            api_key: Some("test_key".to_string()),
            currency: Currency::USD,
            portfolio_update_interval: Duration::from_secs(1800),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&config).unwrap();

        // Deserialize from JSON
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mode, Mode::Live);
        assert_eq!(deserialized.api_key, Some("test_key".to_string()));
        assert_eq!(deserialized.currency, Currency::USD);
        assert_eq!(
            deserialized.portfolio_update_interval,
            Duration::from_secs(1800)
        );
    }

    /// Test Config save and load functionality
    #[test]
    fn test_config_save_and_load() {
        // Backup existing config if it exists
        let config_exists = Path::new(CONFIG_FILE).exists();
        let backup_config = if config_exists {
            let content = fs::read_to_string(CONFIG_FILE).unwrap();
            Some(content)
        } else {
            None
        };

        let test_config = Config {
            mode: Mode::Live,
            api_key: Some("test_api_key".to_string()),
            currency: Currency::EUR,
            portfolio_update_interval: Duration::from_secs(1200),
        };

        // Test save
        let save_result = test_config.save_config();
        assert!(
            save_result.is_ok(),
            "Failed to save config: {:?}",
            save_result.err()
        );

        // Test load
        let loaded_config = Config::load_config();
        assert!(
            loaded_config.is_ok(),
            "Failed to load config: {:?}",
            loaded_config.err()
        );

        let loaded = loaded_config.unwrap();
        assert_eq!(loaded.mode, Mode::Live);
        assert_eq!(loaded.api_key, Some("test_api_key".to_string()));
        assert_eq!(loaded.currency, Currency::EUR);
        assert_eq!(loaded.portfolio_update_interval, Duration::from_secs(1200));

        // Restore original config or clean up
        if let Some(backup) = backup_config {
            fs::write(CONFIG_FILE, backup).unwrap();
        } else {
            let _ = fs::remove_file(CONFIG_FILE);
        }
    }
}
