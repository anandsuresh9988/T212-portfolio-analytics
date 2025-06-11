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
    extract::Form,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task;
use tokio::time::{sleep, Duration};

use crate::{
    models::{
        dividend::DividendInfo,
        portfolio::{Portfolio, Position},
    },
    services::orchestrator::Orchestrator,
    utils::settings::{Config, Mode},
};

#[derive(Template)]
#[template(path = "dividends.html")]
pub struct DividendsTemplate {
    pub dividends: Vec<DividendInfo>,
    pub div_per_year: String,
    pub settings: Config,
}

#[derive(Template)]
#[template(path = "payout.html")]
pub struct PayoutTemplate {
    pub records: Vec<DividendRecord>,
    pub total_dividends: String,
    pub total_wht: String,
    pub ticker_summary: Vec<TickerSummary>,
    pub monthly_div_summary: Vec<(String, f64)>,
    pub settings: Config,
}

#[derive(Template)]
#[template(path = "portfolio.html")]
pub struct PortfolioTemplate {
    pub positions: Vec<Position>,
    pub total_invested: String,
    pub total_current_value: String,
    pub total_pl: String,
    pub last_updated: String,
    pub settings: Config,
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

#[derive(Clone)]
pub struct AppState {
    pub portfolio: Arc<TokioMutex<Portfolio>>,
    pub config: Arc<TokioMutex<Config>>,
    pub tx: mpsc::Sender<()>,
}

// Handler for the dividends page
pub async fn show_dividends(State(state): State<AppState>) -> impl IntoResponse {
    let portfolio = state.portfolio.lock().await;
    let config = state.config.lock().await;
    let mut dividends: Vec<DividendInfo> = portfolio
        .positions
        .iter()
        .filter_map(|pos| pos.div_info.clone())
        .collect();

    let div_per_year: f64 = dividends
        .iter()
        .map(|item| item.annual_income_after_wht)
        .sum();

    dividends.sort_by(|a, b| {
        b.annual_income_after_wht
            .partial_cmp(&a.annual_income_after_wht)
            .unwrap()
    });
    let template = DividendsTemplate {
        dividends,
        div_per_year: format!("{:.2}", div_per_year),
        settings: config.clone(),
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
pub async fn show_portfolio(State(state): State<AppState>) -> impl IntoResponse {
    let portfolio = state.portfolio.lock().await;
    let config = state.config.lock().await;
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
        settings: config.clone(),
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
pub async fn show_payouts(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.lock().await;
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
    let total_dividends: f64 = records
        .iter()
        .filter_map(|r| r.total.parse::<f64>().ok())
        .sum();
    let total_wht: f64 = records
        .iter()
        .filter_map(|r| r.withholding_tax.parse::<f64>().ok())
        .sum();

    // Group by ticker
    let mut ticker_map: HashMap<String, (f64, f64)> = HashMap::new();
    for record in &records {
        if let (Ok(total), Ok(wht)) = (
            record.total.parse::<f64>(),
            record.withholding_tax.parse::<f64>(),
        ) {
            let entry = ticker_map
                .entry(record.ticker.clone())
                .or_insert((0.0, 0.0));
            entry.0 += total;
            entry.1 += wht;
        }
    }

    let mut ticker_summary: Vec<TickerSummary> = ticker_map
        .into_iter()
        .map(|(ticker, (total, wht))| TickerSummary {
            ticker,
            total: format!("{:.2}", total),
            wht: format!("{:.2}", wht),
        })
        .collect();
    ticker_summary.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let monthly_div_summary = calculate_monthly_dividends(&records);

    let template = PayoutTemplate {
        records,
        total_dividends: format!("{:.2}", total_dividends),
        total_wht: format!("{:.2}", total_wht),
        ticker_summary,
        monthly_div_summary,
        settings: config.clone(),
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
pub async fn show_settings(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    let template = SettingsTemplate {
        settings: config.clone(),
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
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsForm {
    api_key: Option<String>,
    currency: String,
    mode: String,
    portfolio_update_interval_secs: u64,
}

pub async fn save_settings(
    State(state): State<AppState>,
    Form(form): Form<UpdateSettingsForm>,
) -> impl IntoResponse {
    let mut config_data = match Config::load_config() {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to load config: {}", e)
                })
                .to_string(),
            )
                .into_response()
        }
    };

    config_data.api_key = form.api_key.clone();
    config_data.currency = form.currency.parse().unwrap_or_default();
    config_data.mode = match form.mode.as_str() {
        "Live" => Mode::Live,
        "Demo" => Mode::Demo,
        _ => Mode::Demo, // Default to Demo if invalid value
    };
    config_data.portfolio_update_interval =
        Duration::from_secs(form.portfolio_update_interval_secs);

    match config_data.save_config() {
        Ok(_) => {
            // Update the shared config
            *state.config.lock().await = config_data;

            // Signal the background task to update immediately
            if let Err(e) = state.tx.send(()).await {
                eprintln!("Failed to signal portfolio update: {}", e);
            }

            (
                StatusCode::OK,
                serde_json::json!({
                    "status": "success",
                    "message": "Settings saved successfully"
                })
                .to_string(),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "status": "error",
                "message": format!("Failed to save config: {}", e)
            })
            .to_string(),
        )
            .into_response(),
    }
}

// Handler to reset settings to default (POST)
pub async fn reset_settings(State(state): State<AppState>, Form(_): Form<()>) -> impl IntoResponse {
    let default_settings = crate::utils::settings::Config::default();

    match default_settings.save_config() {
        Ok(_) => {
            // Update the shared config
            *state.config.lock().await = default_settings;

            // Signal the background task to update immediately
            if let Err(e) = state.tx.send(()).await {
                eprintln!("Failed to signal portfolio update: {}", e);
            }

            (
                StatusCode::OK,
                serde_json::json!({
                    "status": "success",
                    "message": "Settings reset successfully!"
                })
                .to_string(),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "status": "error",
                "message": format!("Error resetting settings: {}", e)
            })
            .to_string(),
        )
            .into_response(),
    }
}

pub async fn start_server(
    portfolio: Portfolio,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let portfolio = Arc::new(TokioMutex::new(portfolio));
    let config = Arc::new(TokioMutex::new(config));

    // Create a channel for signaling immediate updates
    let (tx, mut rx) = mpsc::channel(1);

    let state = AppState {
        portfolio: portfolio.clone(),
        config: config.clone(),
        tx: tx.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Welcome to T212 Portfolio Analytics" }))
        .route(
            "/portfolio",
            get(show_portfolio as fn(axum::extract::State<AppState>) -> _),
        )
        .route(
            "/dividends",
            get(show_dividends as fn(axum::extract::State<AppState>) -> _),
        )
        .route(
            "/payout",
            get(show_payouts as fn(axum::extract::State<AppState>) -> _),
        )
        .route(
            "/settings",
            get(show_settings as fn(axum::extract::State<AppState>) -> _),
        )
        .route(
            "/settings",
            post(
                save_settings as fn(axum::extract::State<AppState>, Form<UpdateSettingsForm>) -> _,
            ),
        )
        .route(
            "/settings/reset",
            post(reset_settings as fn(axum::extract::State<AppState>, Form<()>) -> _),
        )
        .with_state(state);

    // Spawn a background async task to update the portfolio periodically
    let portfolio_for_task = portfolio.clone();
    let config_for_task = config.clone();
    task::spawn(async move {
        loop {
            // Wait for either the update interval or an immediate update signal
            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {
                    // Check if we should update now
                    if let Ok(()) = rx.try_recv() {
                        // Immediate update requested
                        println!("Performing immediate portfolio update");
                    } else {
                        // Get the latest config
                        let current_config = config_for_task.lock().await.clone();
                        // Check if it's time for a regular update
                        if current_config.portfolio_update_interval.as_secs() == 0 {
                            continue;
                        }
                        sleep(current_config.portfolio_update_interval).await;
                    }
                }
                _ = rx.recv() => {
                    // Immediate update requested
                    println!("Performing immediate portfolio update");
                }
            }

            // Get the latest config
            let current_config = config_for_task.lock().await.clone();

            // Create a new portfolio instance and update it
            let mut new_portfolio = Portfolio::default();
            if let Err(e) = new_portfolio.init(&current_config).await {
                eprintln!("Failed to initialize portfolio: {}", e);
                continue;
            }

            // Process the new portfolio data
            let orchestrator = match Orchestrator::new(&current_config).await {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("Failed to create orchestrator: {}", e);
                    continue;
                }
            };

            if let Err(e) = new_portfolio
                .process(
                    &current_config,
                    orchestrator.currency_converter,
                    orchestrator.instrument_metadata,
                )
                .await
            {
                eprintln!("Failed to process portfolio: {}", e);
                continue;
            }

            // Take the lock only briefly to swap in the new data
            {
                let mut shared = portfolio_for_task.lock().await;
                new_portfolio.update_count = shared.update_count + 1;
                *shared = new_portfolio;
                println!("Portfolio update count: {}", shared.update_count);
            }
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
