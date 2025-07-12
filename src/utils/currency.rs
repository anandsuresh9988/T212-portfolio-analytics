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

/// Custom error types for currency conversion operations
#[derive(Debug, Error)]
pub enum CurrencyError {
    /// Error when an invalid currency code is provided
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),
    /// Error when fetching exchange rates from the API fails
    #[error("Failed to fetch exchange rates: {0}")]
    FetchError(String),
    /// Error when a specific exchange rate is not available
    #[error("Rate not available for conversion")]
    RateNotAvailable,
}

/// Supported currency types for the application
///
/// This enum defines the currencies that can be used for conversion.
/// The default currency is GBP (British Pound).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum Currency {
    /// British Pound Sterling - default currency
    #[default]
    GBP,
    /// US Dollar
    USD,
    /// Euro
    EUR,
    /// Swiss Franc
    CHF,
    /// Placeholder for unsupported currencies
    UnSupported,
}

/// Response structure from the exchange rate API
///
/// This struct deserializes the JSON response from the external API
/// that provides current exchange rates relative to a base currency.
#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    /// HashMap containing currency codes as keys and exchange rates as values
    rates: HashMap<String, f64>,
}

/// Main currency converter that manages exchange rates and provides conversion functionality
///
/// This struct maintains a cache of exchange rates and automatically updates them
/// at regular intervals to ensure accuracy. It uses thread-safe shared state
/// to allow concurrent access from multiple parts of the application.
pub struct CurrencyConverter {
    /// Thread-safe cache of exchange rates, keyed by currency code
    rates: Arc<RwLock<HashMap<String, f64>>>,
    /// Timestamp of the last rate update for cache invalidation
    last_update: Arc<RwLock<Instant>>,
    /// Duration between automatic rate updates
    update_interval: Duration,
}

impl CurrencyConverter {
    /// Creates a new CurrencyConverter instance and fetches initial exchange rates
    ///
    /// # Returns
    /// - `Ok(CurrencyConverter)` on successful initialization
    /// - `Err(CurrencyError)` if initial rate fetching fails
    ///
    /// # Example
    /// ```ignore
    /// use t212_portfolio_analytics::utils::currency::Currency;
    ///
    /// let converter = CurrencyConverter::new().await?;
    /// ```
    pub async fn new() -> Result<Self, CurrencyError> {
        let converter = Self {
            rates: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
            update_interval: Duration::from_secs(360),
        };

        // Fetch initial exchange rates
        converter.update_rates().await?;
        Ok(converter)
    }

    /// Fetches the latest exchange rates using the external API
    ///
    /// This method makes an HTTP request to the exchange rate API and updates
    /// the internal cache with the latest rates. The API returns rates relative
    /// to GBP as the base currency.
    ///
    /// # Returns
    /// - `Ok(())` on successful update
    /// - `Err(CurrencyError::FetchError)` if the API request fails
    async fn update_rates(&self) -> Result<(), CurrencyError> {
        let client = reqwest::Client::new();

        // Fetch rates using the exchange rate API (GBP as base currency)
        let response = client
            .get("https://open.er-api.com/v6/latest/GBP")
            .send()
            .await
            .map_err(|e| CurrencyError::FetchError(e.to_string()))?;

        // Deserialize the JSON response into our ExchangeRateResponse struct
        let rates: ExchangeRateResponse = response
            .json()
            .await
            .map_err(|e| CurrencyError::FetchError(e.to_string()))?;

        // Update the cached rates and timestamp
        let mut rates_map = self.rates.write().await;
        println!("Rate = {:?}", rates);
        *rates_map = rates.rates;
        *self.last_update.write().await = Instant::now();

        Ok(())
    }

    /// Ensures that the cached exchange rates are fresh by checking the update interval
    ///
    /// If the rates are older than the update interval, this method will
    /// automatically fetch new rates from the API.
    ///
    /// # Returns
    /// - `Ok(())` if rates are fresh or successfully updated
    /// - `Err(CurrencyError)` if updating rates fails
    async fn ensure_rates_fresh(&self) -> Result<(), CurrencyError> {
        let last_update = *self.last_update.read().await;
        if last_update.elapsed() > self.update_interval {
            self.update_rates().await?;
        }
        Ok(())
    }

    /// Converts an amount from one currency to another using current exchange rates
    ///
    /// This method calculates the conversion factor between two currencies.
    /// If the currencies are the same, it returns 1.0 (no conversion needed).
    ///
    /// # Arguments
    /// - `from`: The source currency
    /// - `to`: The target currency
    ///
    /// # Returns
    /// - `Ok(f64)` containing the conversion factor (multiply source amount by this)
    /// - `Err(CurrencyError::RateNotAvailable)` if either currency rate is not available
    /// - `Err(CurrencyError)` if rate fetching fails
    ///
    /// # Example
    /// ```ignore
    /// use t212_portfolio_analytics::utils::currency::Currency;
    ///
    /// let factor = converter.get_conversion_factor(Currency::USD, Currency::EUR).await?;
    /// let converted_amount = original_amount * factor;
    /// ```
    pub async fn get_conversion_factor(
        &self,
        from: Currency,
        to: Currency,
    ) -> Result<f64, CurrencyError> {
        // No conversion needed if currencies are the same
        if from == to {
            return Ok(1.0);
        }

        // Ensure we have fresh rates
        self.ensure_rates_fresh().await?;
        let rates = self.rates.read().await;

        // Get the exchange rates for both currencies
        let from_rate = rates
            .get(from.as_str())
            .ok_or(CurrencyError::RateNotAvailable)?;
        let to_rate = rates
            .get(to.as_str())
            .ok_or(CurrencyError::RateNotAvailable)?;

        // Calculate conversion factor
        Ok(to_rate / from_rate)
    }
}

/// Implementation of FromStr trait for Currency enum
///
/// Allows parsing currency codes from strings. This is useful for
/// converting user input or API responses into Currency enum values.
impl FromStr for Currency {
    type Err = CurrencyError;

    /// Converts a string to a Currency enum value
    ///
    /// # Arguments
    /// - `s`: The string representation of the currency code
    ///
    /// # Returns
    /// - `Ok(Currency)` for valid currency codes
    /// - `Ok(Currency::UnSupported)` for unrecognized codes
    ///
    /// # Example
    /// ```
    /// use t212_portfolio_analytics::utils::currency::Currency;
    ///
    /// let currency: Currency = "USD".parse().unwrap(); // Ok(Currency::USD)
    /// let currency: Currency = "usd".parse().unwrap(); // Ok(Currency::USD) - case insensitive
    /// ```
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

impl Currency {
    /// Converts the Currency enum to its string representation
    ///
    /// This method returns the standard 3-letter currency code
    /// for each supported currency.
    ///
    /// # Returns
    /// - `&'static str` containing the currency code
    ///
    /// # Example
    /// ```
    /// use t212_portfolio_analytics::utils::currency::Currency;
    ///
    /// assert_eq!(Currency::USD.as_str(), "USD");
    /// assert_eq!(Currency::EUR.as_str(), "EUR");
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_fetch_rates_success() {
        let converter = CurrencyConverter::new().await;
        assert!(
            converter.is_ok(),
            "Failed to create CurrencyConverter: {:?}",
            converter.err()
        );
    }

    #[tokio::test]
    async fn test_conversion_factor_gbp_to_usd() {
        let converter = CurrencyConverter::new()
            .await
            .expect("Failed to create converter");
        let factor = converter
            .get_conversion_factor(Currency::GBP, Currency::USD)
            .await;
        assert!(
            factor.is_ok(),
            "Failed to get conversion factor: {:?}",
            factor.err()
        );
        let factor = factor.unwrap();
        // GBP to USD should be a positive number, may be not the best way of testing this :).
        assert!(
            factor > 0.5 && factor < 2.5,
            "Unexpected GBP->USD factor: {}",
            factor
        );
    }

    #[tokio::test]
    async fn test_conversion_factor_usd_to_eur() {
        let converter = CurrencyConverter::new()
            .await
            .expect("Failed to create converter");
        let factor = converter
            .get_conversion_factor(Currency::USD, Currency::EUR)
            .await;
        assert!(
            factor.is_ok(),
            "Failed to get conversion factor: {:?}",
            factor.err()
        );
        let factor = factor.unwrap();
        // USD to EUR should be a positive number, may be not the best way of testing this :).
        assert!(
            factor > 0.5 && factor < 2.5,
            "Unexpected USD->EUR factor: {}",
            factor
        );
    }
}
