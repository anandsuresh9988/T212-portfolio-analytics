// File: main.rs
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

use t212_portfolio_analytics::models::portfolio::download_export_if_needed;
use t212_portfolio_analytics::models::portfolio::Portfolio;
use t212_portfolio_analytics::services::orchestrator::Orchestrator;
use t212_portfolio_analytics::utils::settings::Config;
use t212_portfolio_analytics::utils::settings::Mode;
use t212_portfolio_analytics::webui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an empty/default portfolio
    let mut portfolio: Portfolio = Portfolio::default();
    let mut config_success: bool = false;

    let config = Config::load_config().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let orchestrator = match Orchestrator::new(&config).await {
        Ok(orchestrator) => {
            println!("Orchestrator initialized successfully");
            config_success = true;
            orchestrator
        }
        Err(e) => {
            eprintln!("Failed to initialize orchestrator: {}", e);
            return Err(e.into());
        }
    };

    if config_success {
        portfolio.init(&config).await?;
        //println!("Instrument metadata {:?}", orchestrator.instrument_metadata);

        if !(config.mode == Mode::Demo) {
            // Try to download export if needed. Payouts are not availabe in Demo mode.
            // So skip this step in Demo mode.
            println!("Downloading export data...");
            download_export_if_needed(&config).await?;
        }

        // Process portfolio with cache file and currency converter
        portfolio
            .process(
                &config,
                orchestrator.currency_converter,
                orchestrator.instrument_metadata,
            )
            .await?;
    }

    // Start the web server
    webui::start_server(portfolio, config, config_success).await?;

    Ok(())
}
