// File: main.rs
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

use t212_portfolio_analytics::models::portfolio::Portfolio;
use t212_portfolio_analytics::services::orchestrator;
use t212_portfolio_analytics::services::orchestrator::Orchestrator;
use t212_portfolio_analytics::services::trading212::RequestType;
use t212_portfolio_analytics::services::trading212::Trading212Client;
use t212_portfolio_analytics::utils::currency::{Currency, CurrencyConverter};
use t212_portfolio_analytics::webui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an empty/default portfolio
    let mut portfolio: Portfolio = Portfolio::default();
    let cache_file = "output.json";

    let orchestrator: Orchestrator = Orchestrator::new().await?;
    portfolio.init().await?;
    //println!("Instrument metadata {:?}", orchestrator.instrument_metadata);

    // Process portfolio with cache file and currency converter
    portfolio.process(
        cache_file,
        orchestrator.currency_converter,
        orchestrator.instrument_metadata,
    )?;

    // Start the web server
    webui::start_server(portfolio).await?;

    Ok(())
}
