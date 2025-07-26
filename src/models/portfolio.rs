// File: portfolio.rs
// Copyright (c) 2025 Anand Sureshkumar
// This file is part of T212 Portfolio Analytics.
// Licensed for personal and educational use only. Commercial use prohibited.
// See the LICENSE file for details.
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
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, fs};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use super::dividend::DividendInfo;
use crate::services::trading212::{DataIncluded, ExportRequest, RequestType, Trading212Client};
use crate::utils::currency::CurrencyConverter;
use crate::utils::settings::{Config, Mode};
use crate::utils::symbol_mapper::extract_symbol;
use crate::{services::trading212::InstrumentMetadata, utils::currency::Currency};

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
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub currency: String,
    pub value: f64,
    pub ppl: f64,    // Profit/Loss
    pub fx_ppl: f64, // FX Profit/Loss
    pub ppl_percent: f64,
    pub div_info: Option<DividendInfo>,
    pub div_prediction: DividendPrediction,
    pub wht: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyPayment {
    pub date: NaiveDate,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividendPrediction {
    pub last_4_dividends_dates: Option<Vec<MonthlyPayment>>,
    pub next_exdate: Option<DateTime<Utc>>,
    pub next_payment_date: Option<DateTime<Utc>>,
    pub payment_amount_per_share: Option<f64>,
    pub net_payment_amount: Option<f64>,
    pub net_wht: Option<f64>,
    pub net_payment_amount_after_wht: Option<f64>,
    pub predicted_monthly_payments: Option<Vec<MonthlyPayment>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Portfolio {
    pub positions: Vec<Position>,
    pub total_value: f64,
    pub total_cost: f64,
    pub total_ppl: f64,
    pub total_ppl_percent: f64,
    pub last_updated: DateTime<Utc>,
    pub update_count: i128,
}

impl Portfolio {
    pub async fn init(&mut self, config: &Config) -> Result<(), anyhow::Error> {
        // Check if we're in Demo mode
        if config.mode == Mode::Demo {
            // Try to load from saved file
            if let Ok(file) = std::fs::File::open("demo_data/demo_positions.json") {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Ok(positions) => {
                        println!("Loaded positions from demo_positions.json");
                        self.positions = positions;
                        return Ok(());
                    }
                    Err(e) => {
                        println!("Failed to parse demo positions data: {}", e);
                        return Err(anyhow::anyhow!("Failed to parse demo positions data"));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("No demo positions data available"));
            }
        } else {
            // Live mode - proceed with Trading212 API
            let trading212_client =
                Trading212Client::new(RequestType::Portfolio, config).map_err(|e| {
                    eprintln!("Trading 212 API error: client initialization failed: {}", e);
                    e
                })?;

            // Fetch open positions
            self.positions = trading212_client.get_open_positions().await.map_err(|e| {
                eprintln!("Trading 212 API error: failed to get open positions: {}", e);
                e
            })?;

            #[cfg(debug_assertions)]
            {
                // Save the data for future use in Demo mode
                if let Ok(file) = std::fs::File::create("demo_positions.json") {
                    let writer = std::io::BufWriter::new(file);
                    if let Err(e) = serde_json::to_writer_pretty(writer, &self.positions) {
                        eprintln!("Failed to save positions data: {}", e);
                    } else {
                        println!("Saved positions data to demo_positions.json");
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn process(
        &mut self,
        config: &Config,
        converter: CurrencyConverter,
        instrument_metadata: Vec<InstrumentMetadata>,
    ) -> Result<(), anyhow::Error> {
        if self.positions.is_empty() {
            println!("No positions are available!");
            return Err(Box::new(PortfolioError::NoPositionsError).into());
        }

        let meta_data_lookup: HashMap<_, _> = instrument_metadata
            .iter()
            .map(|inst| (inst.ticker.clone(), inst.currency_code.clone()))
            .collect();
        // Update vec2 based on the lookup map
        for inst in &mut self.positions {
            if let Some(code) = meta_data_lookup.get(&inst.ticker) {
                inst.currency = (*code).clone();
            }
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

        let mut cache_file = "output.json";
        if config.mode == Mode::Demo {
            cache_file = "demo_data/output.json"
        }
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
        for p in &mut self.positions {
            match parsed.get(p.yf_ticker.clone()) {
                Some(info) => {
                    let yield_opt = info.get("dividendYield").and_then(|v| v.as_f64());
                    let mut rate_opt = info.get("dividendRate").and_then(|v| v.as_f64());

                    p.div_prediction.last_4_dividends_dates = info
                        .get("last_4_dividends")
                        .and_then(|v| v.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(date_str, value)| {
                                    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                                        .ok()
                                        .and_then(|date| {
                                            value
                                                .as_f64()
                                                .map(|v| MonthlyPayment { date, amount: v })
                                        })
                                })
                                .collect()
                        });

                    p.div_prediction.next_payment_date = info.get("dividendDate").and_then(|v| {
                        // dividendDate is always integer (timestamp)
                        v.as_i64()
                            .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                    });

                    p.div_prediction.next_exdate = info.get("exDividendDate").and_then(|v| {
                        // exDividendDate is always integer (timestamp)
                        v.as_i64()
                            .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                    });
                    p.div_prediction.payment_amount_per_share = info
                        .get("corporateActions")
                        .and_then(|arr| arr.get(0))
                        .and_then(|entry| entry.get("meta"))
                        .and_then(|entry| entry.get("amount"))
                        .and_then(|a| {
                            a.as_f64()
                                .or_else(|| a.as_str().and_then(|s| s.parse::<f64>().ok()))
                        });

                    if p.div_prediction.payment_amount_per_share.is_some() {
                        p.div_prediction.net_payment_amount = p
                            .div_prediction
                            .payment_amount_per_share
                            .map(|amt| amt * p.quantity);

                        p.div_prediction.net_wht = p
                            .div_prediction
                            .net_payment_amount
                            .map(|amt| (amt * p.wht) / 100.0);
                        p.div_prediction.net_payment_amount_after_wht = p
                            .div_prediction
                            .net_payment_amount
                            .map(|amt| amt - p.div_prediction.net_wht.unwrap_or(0.0));
                    }

                    if p.currency == "GBX" {
                        p.average_price /= 100.0;
                        p.current_price /= 100.0;
                        p.value /= 100.0;
                    } else {
                        let target_currency = Currency::GBP;
                        let stock_currency =
                            Currency::from_str(&p.currency).unwrap_or(Currency::UnSupported);
                        if stock_currency == Currency::UnSupported {
                            println!(
                                "Add support for currency = {:?} stock = {}",
                                p.currency, p.yf_ticker
                            );
                        } else {
                            let conv_fact = converter
                                .get_conversion_factor(stock_currency, target_currency)
                                .await
                                .unwrap_or(1.00);
                            p.average_price *= conv_fact;
                            p.current_price *= conv_fact;
                            p.value *= conv_fact;
                            rate_opt = rate_opt.map(|rate| rate * conv_fact);
                        }

                        if p.ppl != 0.0 {
                            p.ppl_percent += (p.fx_ppl / p.value) * 100.00;
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
            }
        }

        self.last_updated = Utc::now();
        Ok(())
    }
}

fn calculate_dividend(p: &mut Position, yield_opt: Option<f64>, rate_opt: Option<f64>) {
    let mut annual_dividend_per_share = 0.0;
    if let Some(rate) = rate_opt {
        annual_dividend_per_share = rate;
    } else if let Some(div_yield) = yield_opt {
        annual_dividend_per_share = (div_yield * p.current_price) / 100.0;
    }
    let annual_dividend = annual_dividend_per_share * p.quantity;
    let annual_wht = (annual_dividend * p.wht) / 100.0;
    let annual_income_after_wht = annual_dividend - annual_wht;
    let annual_dividend_per_share_after_wht = annual_dividend_per_share * (100.0 - p.wht) / 100.0;

    let dividend_yield = if p.current_price != 0.0 {
        (annual_dividend_per_share_after_wht / p.current_price) * 100.0
    } else {
        0.0
    };

    let yield_on_cost = if p.average_price != 0.0 {
        (annual_dividend_per_share_after_wht / p.average_price) * 100.0
    } else {
        0.0
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

pub async fn download_export_if_needed(config: &Config) -> Result<(), anyhow::Error> {
    // Check if we already have a recent export
    if std::fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.path().is_file()
                && entry.file_name().to_str().map_or(false, |name| {
                    name.starts_with("export_") && name.ends_with(".csv")
                })
        })
    {
        return Ok(());
    }

    println!("No existing export found. Initiating download from Trading212...");

    // Initialize Trading212 client for exports
    let trading212_client = Trading212Client::new(RequestType::Export, config)
        .map_err(|e| anyhow::anyhow!("Failed to initialize Trading212 client: {}", e))?;

    // Calculate date range for the last year
    let now = Utc::now();
    let one_year_ago = now - chrono::Duration::days(365);

    // Create the export request
    let export_request = ExportRequest {
        data_included: DataIncluded {
            include_dividends: true,
            include_interest: false,
            include_orders: false,
            include_transactions: false,
        },
        time_from: one_year_ago.to_rfc3339(),
        time_to: now.to_rfc3339(),
    };

    println!(
        "Requesting export for period: {} to {}",
        one_year_ago.format("%Y-%m-%d"),
        now.format("%Y-%m-%d")
    );

    // Request new export
    let export_response = trading212_client
        .request_export(&export_request)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to request export: {}", e))?;

    println!("Export initiated with ID: {}", export_response.report_id);

    // Wait and check for export completion
    for attempt in 1..=30 {
        println!("Checking export status (attempt {}/30)...", attempt);
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        if let Some(export_info) = trading212_client
            .get_export_status(export_response.report_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check export status: {}", e))?
        {
            println!("Export status: {}", export_info.status);

            match export_info.status.as_str() {
                "Finished" => {
                    if let Some(download_link) = &export_info.download_link {
                        println!("Export ready! Downloading...");

                        // Download the export
                        let export_data = trading212_client
                            .download_export(download_link)
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to download export: {}", e))?;

                        // Save the export
                        let filename = format!("export_{}.csv", export_info.report_id);
                        std::fs::write(&filename, export_data)
                            .map_err(|e| anyhow::anyhow!("Failed to save export file: {}", e))?;

                        println!("Export saved to {}", filename);
                        return Ok(());
                    }
                }
                "Failed" | "Canceled" => {
                    return Err(anyhow::anyhow!(
                        "Export {} failed or was canceled",
                        export_response.report_id
                    ));
                }
                _ => {
                    println!("Export still processing...");
                }
            }
        } else {
            println!("Export not found in list, waiting...");
        }
    }

    Err(anyhow::anyhow!("Export timed out after 30 attempts"))
}
