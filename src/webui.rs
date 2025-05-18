/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 * 
 * This file is part of the Portfolio Management project.
 */

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use chrono::{Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::models::{dividend::DividendInfo, portfolio::Portfolio};
use crate::services::trading212::{DataIncluded, ExportRequest, RequestType, Trading212Client};
use crate::utils::currency::{Currency, CurrencyConverter};

#[derive(Template)]
#[template(path = "dividends1.html")]
pub struct DividendsTemplate {
    pub dividends: Vec<DividendInfo>,
}

#[derive(Template)]
#[template(path = "payout.html")]
pub struct PayoutTemplate {
    pub records: Vec<DividendRecord>,
    pub total_dividends: String,
    pub total_wht: String,
    pub total_net: String,
    pub ticker_summary: Vec<TickerSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DividendRecord {
    pub date: String,
    pub isin: String,
    pub ticker: String,
    pub name: String,
    pub quantity: String,
    pub price: String,
    pub currency: String,
    pub total: String,
    pub withholding_tax: String,
    pub net_amount: String,
}

#[derive(Debug, Clone)]
pub struct TickerSummary {
    pub ticker: String,
    pub total: String,
    pub wht: String,
    pub net: String,
}

pub async fn get_latest_dividend_records() -> Result<Vec<DividendRecord>, Box<dyn std::error::Error>> {
    // Find the latest export file
    let entries = std::fs::read_dir(".")?;
    let latest_export = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name()?.to_str()?;
                if filename.starts_with("export_") && filename.ends_with(".csv") {
                    return Some(path);
                }
            }
            None
        })
        .max_by_key(|path| path.metadata().ok().and_then(|m| m.modified().ok()))
        .ok_or("No export files found")?;

    let content = std::fs::read_to_string(&latest_export)?;
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let mut records = Vec::new();

    for result in rdr.records() {
        let record = result?;
        if record.len() >= 13 {
            let date = if let Ok(dt) = NaiveDateTime::parse_from_str(&record[1], "%d/%m/%Y %H:%M") {
                dt.format("%Y-%m-%d").to_string()
            } else {
                record[1].to_string()
            };

            let total: f64 = record[9].parse().unwrap_or(0.0);
            let wht: f64 = record[11].parse().unwrap_or(0.0);

            records.push(DividendRecord {
                date,
                isin: record[2].to_string(),
                ticker: record[3].to_string(),
                name: record[4].to_string(),
                quantity: format!("{:.4}", record[5].parse::<f64>().unwrap_or(0.0)),
                price: format!("{:.4}", record[6].parse::<f64>().unwrap_or(0.0)),
                currency: record[7].to_string(),
                total: format!("{:.2}", total),
                withholding_tax: format!("{:.2}", wht),
                net_amount: format!("{:.2}", total - wht),
            });
        }
    }

    // Sort records by date (newest first)
    records.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(records)
}

pub async fn download_export_if_needed() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we already have a recent export
    if let Some(_) = std::fs::read_dir(".")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file()
                && entry.file_name().to_str().map_or(false, |name| {
                    name.starts_with("export_") && name.ends_with(".csv")
                })
        })
        .next()
    {
        return Ok(());
    }

    println!("No existing export found. Initiating download from Trading212...");

    // Load environment variables
    dotenv::dotenv().map_err(|e| format!("Failed to load .env file: {}", e))?;

    // Initialize Trading212 client for exports
    let trading212_client = Trading212Client::new(RequestType::export)
        .map_err(|e| format!("Failed to initialize Trading212 client: {}", e))?;

    // Calculate date range for the last year
    let now = Utc::now();
    let one_year_ago = now - Duration::days(365);

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
        .map_err(|e| format!("Failed to request export: {}", e))?;

    println!("Export initiated with ID: {}", export_response.report_id);

    // Wait and check for export completion
    for attempt in 1..=30 {
        println!("Checking export status (attempt {}/30)...", attempt);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        if let Some(export_info) = trading212_client
            .get_export_status(export_response.report_id)
            .await
            .map_err(|e| format!("Failed to check export status: {}", e))?
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
                            .map_err(|e| format!("Failed to download export: {}", e))?;

                        // Save the export
                        let filename = format!("export_{}.csv", export_info.report_id);
                        std::fs::write(&filename, export_data)
                            .map_err(|e| format!("Failed to save export file: {}", e))?;

                        println!("Export saved to {}", filename);
                        return Ok(());
                    }
                }
                "Failed" | "Canceled" => {
                    return Err(format!(
                        "Export {} failed or was canceled",
                        export_response.report_id
                    )
                    .into());
                }
                _ => {
                    println!("Export still processing...");
                }
            }
        } else {
            println!("Export not found in list, waiting...");
        }
    }

    Err("Export timed out after 30 attempts".into())
}

// Handler for the dividends page
pub async fn show_dividends(State(portfolio): State<Arc<Portfolio>>) -> impl IntoResponse {
    let dividends: Vec<DividendInfo> = portfolio
        .positions
        .iter()
        .filter_map(|pos| pos.div_info.clone())
        .collect();

    let template = DividendsTemplate { dividends };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template rendering error: {}", e),
        )
            .into_response(),
    }
}

// Handler for the payout page
pub async fn show_payouts() -> impl IntoResponse {
    // Try to download export if needed
    if let Err(e) = download_export_if_needed().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to ensure export data: {}", e),
        )
            .into_response();
    }

    // Get the latest export file
    let records = match get_latest_dividend_records().await {
        Ok(records) => records,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error loading dividend records: {}", e),
            )
                .into_response()
        }
    };

    // Calculate totals and summaries
    let mut total_dividends = 0.0;
    let mut total_wht = 0.0;
    let mut ticker_totals: HashMap<String, (f64, f64)> = HashMap::new();

    for record in &records {
        let total: f64 = record.total.parse().unwrap_or(0.0);
        let wht: f64 = record.withholding_tax.parse().unwrap_or(0.0);

        total_dividends += total;
        total_wht += wht;

        let entry = ticker_totals
            .entry(record.ticker.clone())
            .or_insert((0.0, 0.0));
        entry.0 += total;
        entry.1 += wht;
    }

    let total_net = total_dividends - total_wht;

    // Create ticker summary
    let mut ticker_summary: Vec<TickerSummary> = ticker_totals
        .into_iter()
        .map(|(ticker, (total, wht))| TickerSummary {
            ticker,
            total: format!("{:.2}", total),
            wht: format!("{:.2}", wht),
            net: format!("{:.2}", total - wht),
        })
        .collect();

    // Sort ticker summary by total amount descending
    ticker_summary.sort_by(|a, b| {
        b.total
            .parse::<f64>()
            .unwrap_or(0.0)
            .partial_cmp(&a.total.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let template = PayoutTemplate {
        records,
        total_dividends: format!("{:.2}", total_dividends),
        total_wht: format!("{:.2}", total_wht),
        total_net: format!("{:.2}", total_net),
        ticker_summary,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template rendering error: {}", e),
        )
            .into_response(),
    }
}

pub async fn start_server(portfolio: Portfolio) -> Result<(), Box<dyn std::error::Error>> {
    // Wrap portfolio in Arc to share state safely with axum handlers
    let shared_portfolio = Arc::new(portfolio);

    // Build router with shared state
    let app = Router::new()
        .route("/", get(show_dividends))
        .route("/payout", get(show_payouts))
        .with_state(shared_portfolio);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
} 