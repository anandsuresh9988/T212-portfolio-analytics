// File: portfolio.rs
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
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use super::dividend::DividendInfo;
use crate::utils::currency::Currency;
use crate::utils::currency::CurrencyConverter;
use crate::utils::symbol_mapper::extract_symbol;

#[derive(Debug, Error)]
pub enum PortfolioError {
    #[error("No positions are available")]
    NoPositionsError,

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub ticker: String,
    pub yf_ticker: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub currency: String,
    pub value: Decimal,
    pub ppl: Decimal, // Profit/Loss
    pub ppl_percent: Decimal,
    pub div_info: Option<DividendInfo>,
    pub wht: Decimal,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Portfolio {
    pub positions: Vec<Position>,
    pub total_value: Decimal,
    pub total_cost: Decimal,
    pub total_ppl: Decimal,
    pub total_ppl_percent: Decimal,
    pub last_updated: DateTime<Utc>,
}

impl Portfolio {
    pub fn process(
        &mut self,
        cache_file: &str,
        converter: CurrencyConverter,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.positions.is_empty() {
            println!("No positions are available!");
            return Err(Box::new(PortfolioError::NoPositionsError));
        }

        let yfinance_tickers = self
            .positions
            .iter_mut()
            .map(|p| {
                let result = extract_symbol(p.ticker.as_str());
                p.yf_ticker = result.1.yf_ticker.clone();
                p.wht = result.1.tax.into();
                result.1.yf_ticker
            })
            .collect::<Vec<_>>();

        println!("{:?}", yfinance_tickers);

        let json_str = if Path::new(cache_file).exists() {
            // ✅ Read from cache
            println!("Reading from cache...");
            fs::read_to_string(cache_file)?
        } else {
            println!("Fetching details form Yfinance...");
            let output = Command::new("python3")
                .arg("stock_info.py")
                .arg(yfinance_tickers.join(","))
                .output()
                .expect("Failed to run Python script");
            // Check if the Python script ran successfully
            if !output.status.success() {
                eprintln!("Python script failed to run.");
                eprintln!("Exit code: {:?}", output.status.code());
                eprintln!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            let json_output = String::from_utf8_lossy(&output.stdout).to_string();
            fs::write(cache_file, &json_output)?; // ✅ Save to file
            json_output
        };

        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        let _: Vec<_> = self
            .positions
            .iter_mut()
            .map(|p| match parsed.get(p.yf_ticker.clone()) {
                Some(info) => {
                    let yield_opt = info.get("dividendYield").and_then(|v| v.as_f64());
                    let mut rate_opt = info.get("dividendRate").and_then(|v| v.as_f64());

                    let currency = info
                        .get("currency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("NA");

                    if currency == "GBp" {
                        p.average_price /= Decimal::new(100, 0);
                        p.current_price /= Decimal::new(100, 0);
                    } else {
                        let target_currency = Currency::GBP;
                        let stock_currency =
                            Currency::from_str(currency).unwrap_or(Currency::UnSupported);
                        if stock_currency == Currency::UnSupported {
                            println!(
                                "Add support for currency = {:?} stock = {}",
                                currency, p.yf_ticker
                            );
                        } else {
                            let conv_fact = Decimal::from_f64(
                                converter
                                    .convert(1.00, stock_currency, target_currency)
                                    .unwrap_or(1.00),
                            )
                            .unwrap_or(Decimal::new(1, 0));
                            p.average_price *= conv_fact;
                            p.current_price *= conv_fact;
                            rate_opt =
                                rate_opt.map(|rate| rate * conv_fact.to_f64().unwrap_or(1.0));
                        }
                    }

                    if yield_opt.is_some() || rate_opt.is_some() {
                        calculate_dividend(p, yield_opt, rate_opt);
                    } else {
                        println!("Dividend info not available for {}", p.yf_ticker);
                    }
                }
                None => {
                    println!("{} missing in response", p.yf_ticker);
                }
            })
            .collect();
        Ok(())
    }
}

fn calculate_dividend(p: &mut Position, yield_opt: Option<f64>, rate_opt: Option<f64>) {
    let mut annual_dividend_per_share = Decimal::new(0, 0);
    if let Some(rate) = rate_opt {
        annual_dividend_per_share = Decimal::from_f64(rate).unwrap_or_default();
    } else if let Some(div_yield) = yield_opt {
        annual_dividend_per_share =
            (Decimal::from_f64(div_yield).unwrap() * p.current_price) / Decimal::new(100, 0);
    }
    let annual_dividend = annual_dividend_per_share * p.quantity;
    let annual_wht = (annual_dividend * p.wht) / Decimal::new(100, 0);
    let annual_income_after_wht = annual_dividend - annual_wht;
    let annual_dividend_per_share_after_wht =
        annual_dividend_per_share * (Decimal::new(100, 0) - p.wht) / Decimal::new(100, 0);

    let dividend_yield = if !p.current_price.is_zero() {
        (annual_dividend_per_share_after_wht / p.current_price) * Decimal::new(100, 0)
    } else {
        Decimal::new(0, 0)
    };

    let yield_on_cost = if !p.average_price.is_zero() {
        (annual_dividend_per_share_after_wht / p.average_price) * Decimal::new(100, 0)
    } else {
        Decimal::new(0, 0)
    };

    let div_info = DividendInfo {
        symbol: p.yf_ticker.clone(),
        quantity: p.quantity,
        avg_price: p.average_price,
        total_investment: p.quantity * p.average_price,
        annual_dividend_per_share,
        annual_dividend,
        dividend_yield,
        yield_on_cost,
        annual_wht,
        annual_income_after_wht,
        current_investment_val: p.quantity * p.current_price,
    };

    p.div_info = Some(div_info);
    //println!("{:?}", p.div_info);
}
