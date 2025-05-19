/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */

use t212_portfolio_analytics::models::portfolio::Portfolio;
use t212_portfolio_analytics::services::trading212::RequestType;
use t212_portfolio_analytics::services::trading212::Trading212Client;
use t212_portfolio_analytics::utils::currency::{Currency, CurrencyConverter};
use t212_portfolio_analytics::webui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an empty/default portfolio
    let mut portfolio: Portfolio = Portfolio::default();
    let cache_file = "output.json";

    // Load currency rates with USD base
    let converter = CurrencyConverter::load(Currency::USD).await?;

    // Initialize Trading212 client
    let trading212_client = Trading212Client::new(RequestType::portfolio).map_err(|e| {
        eprintln!("Trading 212 API error: client initialization failed: {}", e);
        e
    })?;

    // Fetch open positions
    portfolio.positions = trading212_client.get_open_positions().await.map_err(|e| {
        eprintln!("Trading 212 API error: failed to get open positions: {}", e);
        e
    })?;

    // Process portfolio with cache file and currency converter
    portfolio.process(cache_file, converter)?;

    // Start the web server
    webui::start_server(portfolio).await?;

    Ok(())
}
