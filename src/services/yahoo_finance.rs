// File: yahoo_finance.rs
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

use crate::models::dividend::DividendInfo;
use crate::utils::symbol_mapper::extract_symbol;
use serde::{Deserialize, Serialize};

use thiserror::Error;
use yahoo_finance_api as yahoo;

#[derive(Error, Debug)]
pub enum YahooFinanceError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("No data available for symbol: {0}")]
    NoDataAvailable(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct YahooFinanceResponse {
    #[serde(rename = "quoteSummary")]
    quote_summary: QuoteSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuoteSummary {
    result: Vec<YahooFinanceResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YahooFinanceResult {
    #[serde(rename = "summaryDetail")]
    summary_detail: Option<SummaryDetail>,

    #[serde(rename = "price")]
    price: Option<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SummaryDetail {
    #[serde(rename = "dividendRate")]
    dividend_rate: Option<f64>,

    #[serde(rename = "dividendYield")]
    dividend_yield: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    #[serde(rename = "longName")]
    long_name: Option<String>,

    currency: Option<String>,
}

pub async fn get_stock_info(
    t212_ticker: &str,
    quantity: f64,
    avg_price: f64,
    curr_price: f64,
) -> Result<DividendInfo, YahooFinanceError> {
    let (_orig_ticker, ticker_info) = extract_symbol(t212_ticker);
    let yf_ticker = &ticker_info.yf_ticker;

    let mut provider = yahoo::YahooConnector::new().unwrap();
    let quote_summary = provider
        .get_ticker_info(yf_ticker)
        .await
        .map_err(|e| YahooFinanceError::RequestFailed(e.to_string()));
    //print!("{:?}", quote);

    let mut dividend_rate_dec = 0.0;
    let mut dividend_yield_dec = 0.0;
    let currency = "USD"; // yahoo_finance_api doesn't return currency, so we assume USD or you can maintain mapping
    if let Ok(quote_summary) = quote_summary {
        let dividend_rate = if let Some(summary) = &quote_summary.quote_summary {
            if let Some(summary_data) = summary.result.first() {
                summary_data
                    .summary_detail
                    .as_ref()
                    .and_then(|d| d.dividend_rate)
            } else {
                None
            }
        } else {
            None
        };

        dividend_rate_dec = dividend_rate.unwrap_or(0.00);

        let dividend_yield: Option<f64> = if let Some(summary) = &quote_summary.quote_summary {
            if let Some(summary_data) = summary.result.first() {
                summary_data
                    .summary_detail
                    .as_ref()
                    .and_then(|d| d.dividend_yield)
            } else {
                None
            }
        } else {
            None
        };
        dividend_yield_dec = dividend_yield.unwrap_or(0.00);
    } else {
        println!(
            "Failed to get iyfinace info on ticker = {:?} ",
            (t212_ticker, yf_ticker)
        );
    }

    let cur_conv_fact = match currency {
        "GBP" | "GBp" => 1.0,
        "USD" => 0.79,
        _ => 1.0,
    };

    let yield_on_cost = if avg_price != 0.0 {
        dividend_rate_dec / avg_price
    } else {
        0.0
    };

    let annual_dividend = quantity * dividend_rate_dec * cur_conv_fact;
    let wht = ticker_info.tax as f64 * annual_dividend / 100.0;
    let annual_income_after_wht = annual_dividend - wht;

    let total_investment = if currency == "GBp" {
        quantity * avg_price * cur_conv_fact * 0.01
    } else {
        quantity * avg_price * cur_conv_fact
    };

    let cur_investment = if currency == "GBp" {
        quantity * curr_price * cur_conv_fact * 0.01
    } else {
        quantity * curr_price * cur_conv_fact
    };

    Ok(DividendInfo {
        symbol: yf_ticker.to_string(),
        quantity,
        avg_price,
        total_investment,
        annual_dividend_per_share: dividend_rate_dec,
        annual_dividend,
        dividend_yield: dividend_yield_dec * 100.0,
        yield_on_cost: yield_on_cost * 100.0,
        annual_wht: wht,
        annual_income_after_wht,
        current_investment_val: cur_investment,
    })
}
