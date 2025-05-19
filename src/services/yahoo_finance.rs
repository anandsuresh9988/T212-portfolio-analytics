/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */
use crate::models::dividend::DividendInfo;
use crate::utils::symbol_mapper::extract_symbol;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result;
use thiserror::Error;
use yahoo_finance_api as yahoo;

use chrono::Utc;
use rust_decimal::prelude::*;

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
    dividend_rate: Option<ValueWrapper>,

    #[serde(rename = "dividendYield")]
    dividend_yield: Option<ValueWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    #[serde(rename = "longName")]
    long_name: Option<String>,

    currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValueWrapper {
    raw: Option<f64>,
}

pub async fn get_stock_info(
    t212_ticker: &str,
    quantity: Decimal,
    avg_price: Decimal,
    curr_price: Decimal,
) -> Result<DividendInfo, YahooFinanceError> {
    let (orig_ticker, ticker_info) = extract_symbol(t212_ticker);
    let yf_ticker = &ticker_info.yf_ticker;

    let mut provider = yahoo::YahooConnector::new().unwrap();
    let quote_summary = provider
        .get_ticker_info(yf_ticker)
        .await
        .map_err(|e| YahooFinanceError::RequestFailed(e.to_string()));
    //print!("{:?}", quote);

    let mut dividend_rate_dec = Decimal::ZERO;
    let mut dividend_yield_dec = Decimal::ZERO;
    let currency = "USD"; // yahoo_finance_api doesn't return currency, so we assume USD or you can maintain mapping
    if let Ok(quote_summary) = quote_summary {
        let dividend_rate = if let Some(summary) = &quote_summary.quote_summary {
            if let Some(summary_data) = summary.result.get(0) {
                summary_data
                    .summary_detail
                    .as_ref()
                    .and_then(|d| d.dividend_rate.as_ref())
                    .and_then(|d| d.to_f64())
            } else {
                None
            }
        } else {
            None
        };

        dividend_rate_dec =
            Decimal::from_f64(dividend_rate.unwrap_or(0.00)).unwrap_or(Decimal::ZERO);

        let dividend_yield: Option<f64> = if let Some(summary) = &quote_summary.quote_summary {
            if let Some(summary_data) = summary.result.get(0) {
                summary_data
                    .summary_detail
                    .as_ref()
                    .and_then(|d| d.dividend_yield.as_ref())
                    .and_then(|d| d.to_f64())
            } else {
                None
            }
        } else {
            None
        };
        dividend_yield_dec =
            Decimal::from_f64(dividend_yield.unwrap_or(0.00)).unwrap_or(Decimal::ZERO);
    } else {
        println!(
            "Failed to get iyfinace info on ticker = {:?} ",
            (t212_ticker, yf_ticker)
        );
    }

    let cur_conv_fact = match currency {
        "GBP" | "GBp" => Decimal::ONE,
        "USD" => Decimal::from_f64(0.79).unwrap_or_default(),
        _ => Decimal::ONE,
    };

    let yield_on_cost = if avg_price != Decimal::ZERO {
        dividend_rate_dec / avg_price
    } else {
        Decimal::ZERO
    };

    let annual_dividend = quantity * dividend_rate_dec * cur_conv_fact;
    let wht = Decimal::from(ticker_info.tax) * annual_dividend / Decimal::from(100);
    let annual_income_after_wht = annual_dividend - wht;

    let total_investment = if currency == "GBp" {
        quantity * avg_price * cur_conv_fact * Decimal::from_f64(0.01).unwrap_or_default()
    } else {
        quantity * avg_price * cur_conv_fact
    };

    let cur_investment = if currency == "GBp" {
        quantity * curr_price * cur_conv_fact * Decimal::from_f64(0.01).unwrap_or_default()
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
        dividend_yield: dividend_yield_dec * Decimal::from(100),
        yield_on_cost: yield_on_cost * Decimal::from(100),
        annual_wht: wht,
        annual_income_after_wht,
        current_investment_val: cur_investment,
    })
}
