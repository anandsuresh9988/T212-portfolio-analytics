/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeRateError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Conversion rate not available: {0} to {1}")]
    ConversionNotAvailable(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ExchangeRateResponse {
    result: String,
    rates: HashMap<String, f64>,
}

struct CachedRates {
    base: String,
    rates: HashMap<String, f64>,
    timestamp: DateTime<Utc>,
}

static CACHED_RATES: Lazy<RwLock<Option<CachedRates>>> = Lazy::new(|| RwLock::new(None));

pub async fn get_conversion_rate(
    from_currency: &str,
    to_currency: &str,
) -> Result<f64, ExchangeRateError> {
    let from_currency = from_currency.to_uppercase();
    let to_currency = to_currency.to_uppercase();

    // Self-conversion
    if from_currency == to_currency {
        return Ok(1.0);
    }

    // Check cache first
    {
        let cache = CACHED_RATES.read().unwrap();
        if let Some(cached) = &*cache {
            if cached.base == from_currency
                && cached.timestamp + Duration::hours(1) > Utc::now()
                && cached.rates.contains_key(&to_currency)
            {
                return Ok(*cached.rates.get(&to_currency).unwrap());
            }
        }
    }

    // Need to fetch new rates
    let url = format!("https://open.er-api.com/v6/latest/{}", from_currency);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ExchangeRateError::RequestFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ExchangeRateError::RequestFailed(format!(
            "API returned status code: {}",
            response.status()
        )));
    }

    let data: ExchangeRateResponse = response
        .json()
        .await
        .map_err(|e| ExchangeRateError::ParseError(e.to_string()))?;

    if data.result != "success" {
        return Err(ExchangeRateError::RequestFailed(
            "API returned non-success result".to_string(),
        ));
    }

    if !data.rates.contains_key(&to_currency) {
        return Err(ExchangeRateError::ConversionNotAvailable(
            from_currency.clone(),
            to_currency.clone(),
        ));
    }

    let rate = data.rates[&to_currency];

    // Update cache
    {
        let mut cache = CACHED_RATES.write().unwrap();
        *cache = Some(CachedRates {
            base: from_currency.clone(),
            rates: data.rates,
            timestamp: Utc::now(),
        });
    }

    Ok(rate)
}
