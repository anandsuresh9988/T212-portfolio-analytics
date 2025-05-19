/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */
use crate::models::portfolio::{Portfolio, Position};
use reqwest::header::{HeaderMap, HeaderValue};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Trading212Error {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Missing API key")]
    MissingApiKey,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trading212Position {
    ticker: String,
    quantity: f64,
    averagePrice: f64,
    currentPrice: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    #[serde(rename = "dataIncluded")]
    pub data_included: DataIncluded,
    #[serde(rename = "timeFrom")]
    pub time_from: String,
    #[serde(rename = "timeTo")]
    pub time_to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataIncluded {
    #[serde(rename = "includeDividends")]
    pub include_dividends: bool,
    #[serde(rename = "includeInterest")]
    pub include_interest: bool,
    #[serde(rename = "includeOrders")]
    pub include_orders: bool,
    #[serde(rename = "includeTransactions")]
    pub include_transactions: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExportResponse {
    #[serde(rename = "reportId")]
    pub report_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ExportInfo {
    #[serde(rename = "dataIncluded")]
    pub data_included: DataIncluded,
    #[serde(rename = "downloadLink")]
    pub download_link: Option<String>,
    #[serde(rename = "reportId")]
    pub report_id: i64,
    pub status: String,
    #[serde(rename = "timeFrom")]
    pub time_from: String,
    #[serde(rename = "timeTo")]
    pub time_to: String,
}

pub struct Trading212Client {
    pub client: reqwest::Client,
    pub base_url: String,
    pub headers: HeaderMap,
}

#[derive(PartialEq)]
pub enum RequestType {
    portfolio,
    dividends_paid,
    export,
}

impl Trading212Client {
    pub fn new(rqst_type: RequestType) -> Result<Self, Trading212Error> {
        let api_key =
            env::var("TRADING212_API_TOKEN").map_err(|_| Trading212Error::MissingApiKey)?;
        let mut base_url = "".to_string();

        let target = env::var("T212_TARGET").unwrap_or_else(|_| "live".to_string());
        match rqst_type {
            RequestType::portfolio => {
                base_url = if target == "live" {
                    "https://live.trading212.com/api/v0/equity/portfolio".to_string()
                } else {
                    "https://demo.trading212.com/api/v0/equity/portfolio".to_string()
                };
            }
            RequestType::dividends_paid => {
                base_url = if target == "live" {
                    "https://live.trading212.com/api/v0/history/dividends".to_string()
                } else {
                    "https://demo.trading212.com/api/v0/history/dividends".to_string()
                };
            }
            RequestType::export => {
                base_url = if target == "live" {
                    "https://live.trading212.com/api/v0/history/exports".to_string()
                } else {
                    "https://demo.trading212.com/api/v0/history/exports".to_string()
                };
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&api_key)
                .map_err(|e| Trading212Error::RequestFailed(e.to_string()))?,
        );
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("trading212-dividend-analyzer/0.1.0"),
        );

        let client = reqwest::Client::new();

        Ok(Self {
            client,
            base_url,
            headers,
        })
    }

    pub async fn get_open_positions(&self) -> Result<Vec<Position>, Trading212Error> {
        let response = self
            .client
            .get(&self.base_url)
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| Trading212Error::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Trading212Error::RequestFailed(format!(
                "API returned status code: {}",
                response.status()
            )));
        }

        let positions: Vec<Trading212Position> = response
            .json()
            .await
            .map_err(|e| Trading212Error::ParseError(e.to_string()))?;

        let positions = positions
            .into_iter()
            .filter(|p| p.quantity > 0.0)
            .map(|p| Position {
                ticker: p.ticker,
                quantity: Decimal::from_f64(p.quantity).unwrap_or_default(),
                average_price: Decimal::from_f64(p.averagePrice).unwrap_or_default(),
                current_price: Decimal::from_f64(p.currentPrice).unwrap_or_default(),
                currency: "".to_string(), // Will be populated later
                value: Decimal::from_f64(p.quantity * p.currentPrice).unwrap_or_default(),
                ppl: Decimal::from_f64(p.quantity * (p.currentPrice - p.averagePrice))
                    .unwrap_or_default(),
                ppl_percent: if p.averagePrice > 0.0 {
                    Decimal::from_f64((p.currentPrice / p.averagePrice - 1.0) * 100.0)
                        .unwrap_or_default()
                } else {
                    Decimal::from_f64(0.0).unwrap_or_default()
                },
                div_info: None,
                yf_ticker: "".to_string(),
                WHT: Decimal::from_f64(0.0).unwrap_or_default(),
            })
            .collect();

        Ok(positions)
    }

    pub async fn request_export(
        &self,
        request: &ExportRequest,
    ) -> Result<ExportResponse, Trading212Error> {
        println!("Sending export request to: {}", self.base_url);
        let response = self
            .client
            .post(&self.base_url)
            .headers(self.headers.clone())
            .json(request)
            .send()
            .await
            .map_err(|e| Trading212Error::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Trading212Error::RequestFailed(format!(
                "API returned status code: {} - {}",
                status, error_text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Trading212Error::ParseError(e.to_string()))
    }

    pub async fn get_export_status(
        &self,
        report_id: i64,
    ) -> Result<Option<ExportInfo>, Trading212Error> {
        println!("Checking export status at: {}", self.base_url);
        let response = self
            .client
            .get(&self.base_url)
            .headers(self.headers.clone())
            .send()
            .await
            .map_err(|e| Trading212Error::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Trading212Error::RequestFailed(format!(
                "API returned status code: {} - {}",
                status, error_text
            )));
        }

        let exports: Vec<ExportInfo> = response
            .json()
            .await
            .map_err(|e| Trading212Error::ParseError(e.to_string()))?;

        Ok(exports.into_iter().find(|e| e.report_id == report_id))
    }

    pub async fn download_export(&self, download_link: &str) -> Result<String, Trading212Error> {
        println!("Downloading export from: {}", download_link);
        let client = reqwest::Client::new();
        let response = client
            .get(download_link)
            .send()
            .await
            .map_err(|e| Trading212Error::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Trading212Error::RequestFailed(format!(
                "Download failed with status code: {} - {}",
                status, error_text
            )));
        }

        response
            .text()
            .await
            .map_err(|e| Trading212Error::RequestFailed(e.to_string()))
    }
}
