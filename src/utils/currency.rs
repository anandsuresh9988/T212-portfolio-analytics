/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */

use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use reqwest;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    INR,
    UnSupported,
}

#[derive(Debug)]
pub struct ParseCurrencyError;

impl fmt::Display for ParseCurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid currency code")
    }
}

impl Error for ParseCurrencyError {}

impl FromStr for Currency {
    type Err = ParseCurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            "INR" => Ok(Currency::INR),
            _ => Err(ParseCurrencyError),
        }
    }
}

impl TryFrom<&str> for Currency {
    type Error = ParseCurrencyError;

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
            Currency::INR => "INR",
            _ => "UnSupported",
        }
    }
}

#[derive(Debug)]
pub struct CurrencyConverter {
    rates: HashMap<Currency, f64>,
    base: Currency,
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}
impl CurrencyConverter {
    /// Load live exchange rates for given base currency asynchronously (using Frankfurter API)
    pub async fn load(base: Currency) -> Result<Self, Box<dyn std::error::Error>> {
        // Frankfurter API URL format: https://api.frankfurter.app/latest?from=USD
        let url = format!("https://api.frankfurter.app/latest?from={}", base.as_str());

        let resp = reqwest::get(&url).await?;
        let text = resp.text().await?;
        //println!("API response: {}", text); // Debug print

        // Deserialize FrankfurterResponse
        let data: FrankfurterResponse = serde_json::from_str(&text)?;

        let mut rates = std::collections::HashMap::new();

        // Base currency rate is always 1.0
        rates.insert(base.clone(), 1.0);

        for (code, rate) in data.rates {
            if let Ok(curr) = Currency::try_from(code.as_str()) {
                rates.insert(curr, rate);
            }
        }

        Ok(CurrencyConverter { rates, base })
    }

    /// Convert amount from one currency to another using loaded rates
    pub fn convert(&self, amount: f64, from: Currency, to: Currency) -> Option<f64> {
        if from == to {
            return Some(amount);
        }

        let from_rate = self.rates.get(&from)?;
        let to_rate = self.rates.get(&to)?;

        Some(amount / from_rate * to_rate)
    }
}
