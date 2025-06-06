// File: webui.rs
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

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time::{sleep, Duration};

use crate::services::trading212::{DataIncluded, ExportRequest, RequestType, Trading212Client};
use crate::{
    models::{
        dividend::DividendInfo,
        portfolio::{Portfolio, Position},
    },
    services::orchestrator::Orchestrator,
    utils::settings::Config,
};

#[derive(Template)]
#[template(path = "dividends.html")]
pub struct DividendsTemplate {
    pub dividends: Vec<DividendInfo>,
    pub div_per_year: String,
}

#[derive(Template)]
#[template(path = "payout.html")]
pub struct PayoutTemplate {
    pub records: Vec<DividendRecord>,
    pub total_dividends: String,
    pub total_wht: String,
    pub ticker_summary: Vec<TickerSummary>,
    pub monthly_div_summary: Vec<(String, f64)>,
}

#[derive(Template)]
#[template(path = "portfolio.html")]
pub struct PortfolioTemplate {
    pub positions: Vec<Position>,
    pub total_invested: String,
    pub total_current_value: String,
    pub total_pl: String,
    pub last_updated: String,
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    pub settings: Config,
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
}

#[derive(Debug, Clone)]
pub struct TickerSummary {
    pub ticker: String,
    pub total: String,
    pub wht: String,
}

pub fn calculate_monthly_dividends(records: &[DividendRecord]) -> Vec<(String, f64)> {
    let mut monthly_sums: HashMap<String, f64> = HashMap::new();

    for record in records {
        if let Ok(date) = NaiveDate::parse_from_str(&record.date, "%Y-%m-%d %H:%M:%S") {
            let month_name = date.format("%b %Y").to_string(); // "Feb 2025"
            if let Ok(amount) = record.total.parse::<f64>() {
                *monthly_sums.entry(month_name).or_insert(0.0) += amount;
            }
        }
    }
    // Convert the HashMap into a sorted Vec of tuples
    let mut result: Vec<(String, f64)> = monthly_sums.into_iter().collect();
    result.sort_by_key(|(month, _)| {
        NaiveDate::parse_from_str(&(month.to_string() + " 01"), "%b %Y %d").unwrap_or_default()
    });
    result.reverse();

    result
}
pub async fn get_latest_dividend_records() -> Result<Vec<DividendRecord>, Box<dyn std::error::Error>>
{
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
            });
        }
    }

    // Sort records by date (newest first)
    records.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(records)
}

// Handler for the dividends page
pub async fn show_dividends(State(portfolio): State<Arc<Mutex<Portfolio>>>) -> impl IntoResponse {
    let portfolio = portfolio.lock().unwrap();
    let dividends: Vec<DividendInfo> = portfolio
        .positions
        .iter()
        .filter_map(|pos| pos.div_info.clone())
        .collect();

    let div_per_year: f64 = dividends
        .iter()
        .map(|item| item.annual_income_after_wht)
        .sum();

    let template = DividendsTemplate {
        dividends,
        div_per_year: format!("{:.2}", div_per_year),
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

// Handler for the dividends page
pub async fn show_portfolio(State(portfolio): State<Arc<Mutex<Portfolio>>>) -> impl IntoResponse {
    let portfolio = portfolio.lock().unwrap();
    let positions = &portfolio.positions;
    let total_invested: f64 = portfolio
        .positions
        .iter()
        .map(|p| p.average_price * p.quantity)
        .sum();
    let total_current_value: f64 = portfolio.positions.iter().map(|p| p.value).sum();
    let total_pl: f64 = portfolio.positions.iter().map(|p| p.ppl).sum();

    let template = PortfolioTemplate {
        positions: positions.to_vec(),
        total_invested: format!("{:.2}", total_invested),
        total_current_value: format!("{:.2}", total_current_value),
        total_pl: format!("{:.2}", total_pl),
        last_updated: portfolio
            .last_updated
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
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

// Handler for the payout page
pub async fn show_payouts() -> impl IntoResponse {
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

    // Create ticker summary
    let mut ticker_summary: Vec<TickerSummary> = ticker_totals
        .into_iter()
        .map(|(ticker, (total, wht))| TickerSummary {
            ticker,
            total: format!("{:.2}", total),
            wht: format!("{:.2}", wht),
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

    let monthly_dividends = calculate_monthly_dividends(&records);

    let template = PayoutTemplate {
        records,
        total_dividends: format!("{:.2}", total_dividends),
        total_wht: format!("{:.2}", total_wht),
        ticker_summary,
        monthly_div_summary: monthly_dividends,
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

// Handler for the settings page (GET)
pub async fn show_settings() -> impl IntoResponse {
    let settings = Config::load_config().unwrap_or_else(|_| Config::default());

    let template = SettingsTemplate {
        settings: settings.clone(),
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

// Handler for the settings page (POST)
#[derive(Deserialize)]
pub struct UpdateSettingsForm {
    api_key: Option<String>,
    currency: String,
    portfolio_update_interval_secs: u64,
}

pub async fn save_settings(form: axum::extract::Form<UpdateSettingsForm>) -> impl IntoResponse {
    let mut settings = Config::load_config().unwrap_or_else(|_| Config::default());

    settings.api_key = form.api_key.clone();
    settings.currency = form
        .currency
        .parse()
        .unwrap_or(crate::utils::currency::Currency::UnSupported);
    settings.portfolio_update_interval = Duration::from_secs(form.portfolio_update_interval_secs);

    match settings.save_config() {
        Ok(_) => axum::Json(serde_json::json!({
            "status": "success",
            "message": "Settings saved successfully!"
        }))
        .into_response(),
        Err(e) => axum::Json(serde_json::json!({
            "status": "error",
            "message": format!("Error saving settings: {}", e)
        }))
        .into_response(),
    }
}

// Handler to reset settings to default (POST)
pub async fn reset_settings() -> impl IntoResponse {
    let default_settings = crate::utils::settings::Config::default();

    match default_settings.save_config() {
        Ok(_) => axum::Json(serde_json::json!({
            "status": "success",
            "message": "Settings reset successfully!"
        }))
        .into_response(),
        Err(e) => axum::Json(serde_json::json!({
            "status": "error",
            "message": format!("Error resetting settings: {}", e)
        }))
        .into_response(),
    }
}

pub async fn start_server(
    portfolio: Portfolio,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let shared_portfolio = Arc::new(Mutex::new(portfolio));

    let app = Router::new()
        .route("/", get(show_portfolio))
        .route("/portfolio", get(show_portfolio))
        .route("/dividends", get(show_dividends))
        .route("/payout", get(show_payouts))
        .route("/settings", get(show_settings).post(save_settings))
        .route("/settings/reset", post(reset_settings))
        .with_state(shared_portfolio.clone());

    // Spawn a background async task to update the portfolio periodically
    let portfolio_for_task = shared_portfolio.clone();
    task::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            // Create a new portfolio instance and update it
            let mut new_portfolio = Portfolio::default();
            if let Err(e) = new_portfolio.init(&config).await {
                eprintln!("Failed to initialize portfolio: {}", e);
            }

            // Process the new portfolio data
            let orchestrator = Orchestrator::new(&config).await.unwrap();
            if let Err(e) = new_portfolio.process(
                "cache.json",
                orchestrator.currency_converter,
                orchestrator.instrument_metadata,
            ) {
                eprintln!("Failed to process portfolio: {}", e);
            }

            // Take the lock only briefly to swap in the new data
            {
                let mut shared = portfolio_for_task.lock().unwrap();
                new_portfolio.update_count = shared.update_count + 1;
                *shared = new_portfolio;
                println!("Portfolio update count: {}", shared.update_count);
            }

            // Wait 15 minutes before next update
            sleep(Duration::from_secs(60)).await;
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
