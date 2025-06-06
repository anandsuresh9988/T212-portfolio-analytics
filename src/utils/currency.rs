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

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CurrencyError {
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum Currency {
    #[default]
    GBP,
    USD,
    EUR,
    UnSupported,
}

impl FromStr for Currency {
    type Err = CurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GBP" => Ok(Currency::GBP),
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
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
            _ => "UnSupported",
        }
    }
}

pub struct CurrencyConverter {
    gbp_to_usd: f64,
    gbp_to_eur: f64,
}

impl CurrencyConverter {
    pub fn new(gbp_to_usd: f64, gbp_to_eur: f64) -> Self {
        Self {
            gbp_to_usd,
            gbp_to_eur,
        }
    }

    pub fn convert(&self, amount: f64, from: Currency, to: Currency) -> Option<f64> {
        if from == to {
            return Some(amount);
        }

        match (from, to) {
            (Currency::GBP, Currency::USD) => Some(amount * self.gbp_to_usd),
            (Currency::GBP, Currency::EUR) => Some(amount * self.gbp_to_eur),
            (Currency::USD, Currency::GBP) => Some(amount / self.gbp_to_usd),
            (Currency::EUR, Currency::GBP) => Some(amount / self.gbp_to_eur),
            (Currency::USD, Currency::EUR) => Some(amount * self.gbp_to_eur / self.gbp_to_usd),
            (Currency::EUR, Currency::USD) => Some(amount * self.gbp_to_usd / self.gbp_to_eur),
            _ => None,
        }
    }
}
