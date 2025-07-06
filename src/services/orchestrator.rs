// File: orchestrator.rs
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

use crate::services::trading212::{InstrumentMetadata, RequestType, Trading212Client};
use crate::utils::currency::CurrencyConverter;
use crate::utils::settings::Config;
use crate::utils::settings::Mode;
use serde_json;

pub struct Orchestrator {
    pub currency_converter: CurrencyConverter,
    pub instrument_metadata: Vec<InstrumentMetadata>,
}

impl Orchestrator {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize Trading212 client for metadata
        let instrument_metadata = if config.mode == Mode::Demo {
            // Try to load from saved file
            if let Ok(file) = std::fs::File::open("./demo_data/demo_instruments.json") {
                let reader = std::io::BufReader::new(file);
                if let Ok(metadata) = serde_json::from_reader(reader) {
                    println!("Loaded instruments metadata from demo_instruments.json");
                    metadata
                } else {
                    return Err("Failed to parse demo instruments data".into());
                }
            } else {
                return Err("No demo instruments data available".into());
            }
        } else {
            // Live mode - proceed with Trading212 API
            let trading212_client =
                Trading212Client::new(RequestType::InstrumentsMetadata, config)?;

            // Get instrument metadata
            let metadata = trading212_client.get_instruments_metadata().await?;

            // Save the data for future use in Demo mode
            if let Ok(file) = std::fs::File::create("demo_instruments.json") {
                let writer = std::io::BufWriter::new(file);
                if let Err(e) = serde_json::to_writer_pretty(writer, &metadata) {
                    eprintln!("Failed to save instruments metadata: {}", e);
                } else {
                    println!("Saved instruments metadata to demo_instruments.json");
                }
            }

            metadata
        };

        // Create currency converter with fixed rates
        // These rates should be updated periodically in a real application
        let currency_converter = CurrencyConverter::new().await?;

        Ok(Self {
            currency_converter,
            instrument_metadata,
        })
    }
}
