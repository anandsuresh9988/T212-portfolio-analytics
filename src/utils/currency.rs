// File: currency.rs
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

use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum CurrencyError {
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),
    #[error("Failed to fetch exchange rates: {0}")]
    FetchError(String),
    #[error("Rate not available for conversion")]
    RateNotAvailable,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum Currency {
    #[default]
    GBP,
    USD,
    EUR,
    CHF,
    UnSupported,
}

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    rates: HashMap<String, f64>,
}

pub struct CurrencyConverter {
    rates: Arc<RwLock<HashMap<String, f64>>>,
    last_update: Arc<RwLock<Instant>>,
    update_interval: Duration,
}

impl CurrencyConverter {
    pub async fn new() -> Result<Self, CurrencyError> {
        let converter = Self {
            rates: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
            update_interval: Duration::from_secs(360), // Update every hour
        };

        // Initial fetch
        converter.update_rates().await?;
        Ok(converter)
    }

    async fn update_rates(&self) -> Result<(), CurrencyError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://open.er-api.com/v6/latest/GBP")
            .send()
            .await
            .map_err(|e| CurrencyError::FetchError(e.to_string()))?;

        let rates: ExchangeRateResponse = response
            .json()
            .await
            .map_err(|e| CurrencyError::FetchError(e.to_string()))?;

        let mut rates_map = self.rates.write().await;
        println!("Rate = {:?}", rates);
        *rates_map = rates.rates;
        *self.last_update.write().await = Instant::now();

        Ok(())
    }

    async fn ensure_rates_fresh(&self) -> Result<(), CurrencyError> {
        let last_update = *self.last_update.read().await;
        if last_update.elapsed() > self.update_interval {
            self.update_rates().await?;
        }
        Ok(())
    }

    pub async fn convert(
        &self,
        amount: f64,
        from: Currency,
        to: Currency,
    ) -> Result<f64, CurrencyError> {
        if from == to {
            return Ok(amount);
        }

        self.ensure_rates_fresh().await?;
        let rates = self.rates.read().await;

        let from_rate = rates
            .get(from.as_str())
            .ok_or(CurrencyError::RateNotAvailable)?;
        let to_rate = rates
            .get(to.as_str())
            .ok_or(CurrencyError::RateNotAvailable)?;

        // Convert to GBP first (base currency), then to target currency
        let amount_in_gbp = amount / from_rate;
        Ok(amount_in_gbp * to_rate)
    }
}

impl FromStr for Currency {
    type Err = CurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GBP" => Ok(Currency::GBP),
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "CHF" => Ok(Currency::CHF),
            _ => Ok(Currency::UnSupported),
        }
    }
}

impl TryFrom<&str> for Currency {
    type Error = CurrencyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Currency::from_str(value)
    }
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::CHF => "CHF",
            _ => "UnSupported",
        }
    }
}
