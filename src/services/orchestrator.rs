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

use crate::services::trading212::RequestType;
use crate::utils::currency::Currency;
use crate::{services::trading212::InstrumentMetadata, utils::currency::CurrencyConverter};

use super::trading212::Trading212Client;

pub struct Orchestrator {
    pub currency_converter: CurrencyConverter,
    pub instrument_metadata: Vec<InstrumentMetadata>,
}

impl Orchestrator {
    pub async fn new() -> Result<Orchestrator, Box<dyn std::error::Error>> {
        // Initialize Trading212 client
        let trading212_client_metadata = Trading212Client::new(RequestType::InstrumentsMetadata)
            .map_err(|e| {
                eprintln!("Trading 212 API error: client initialization failed: {}", e);
                e
            })?;

        // Fetch instrument metadata
        let instrument_metatdata = trading212_client_metadata
            .get_instruments_metadata()
            .await
            .map_err(|e| {
                eprintln!(
                    "Trading 212 API error: failed to get instrument metadata: {}",
                    e
                );
                e
            })?;

        // Load currency rates with USD base
        let converter = CurrencyConverter::load(Currency::USD).await?;

        Ok(Orchestrator {
            currency_converter: converter,
            instrument_metadata: instrument_metatdata,
        })
    }
}
