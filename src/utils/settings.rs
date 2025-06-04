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

use std::{fs::File, io::{BufReader, BufWriter}, time::Duration};

use serde::{Deserialize, Serialize};

use super::currency::Currency;

const DEFAULT_PORTFOLIO_UPDATE_TIME_S: u64 = 60*60;
const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    currency: Currency,
    #[serde(default = "default_timeout" )]
    portfolio_update_interval: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_PORTFOLIO_UPDATE_TIME_S)
}

impl Config {
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(CONFIG_FILE)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    pub fn load_config() -> Result<Self, Box<dyn std::error::Error>>{
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}