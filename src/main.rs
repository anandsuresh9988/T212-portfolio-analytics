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

use log;
use t212_portfolio_analytics::models::portfolio::Portfolio;
use t212_portfolio_analytics::services::orchestrator::Orchestrator;
use t212_portfolio_analytics::utils::settings::Config;
use t212_portfolio_analytics::webui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::info!("Starting T212 Portfolio Analytics...");

    // Create an default portfolio. This will be empty and will
    // be populated later during processing of individual positions
    let mut portfolio: Portfolio = Portfolio::default();
    #[allow(unused_assignments)]
    let mut config_success: bool = false;

    let config = Config::load_config()?;

    let orchestrator = match Orchestrator::new(&config).await {
        Ok(orchestrator) => {
            println!("Orchestrator initialized successfully");
            config_success = true;
            orchestrator
        }
        Err(e) => {
            eprintln!("Failed to initialize orchestrator: {}", e);
            return Err(anyhow::Error::msg(e.to_string()));
        }
    };

    if config_success {
        // Only initialize the portfolio if the orchestrator was created successfully.
        // If orchestrator creation fails, this will be fetched later in the processing thread
        // This will fetch all the T212 positions and create placeholders in the portfolio.
        println!("Initializing portfolio...");
        portfolio.init(&config).await?;

        // Process portfolio. This stage will fetch other information for processing each
        // positions, like the yahoo finance data.
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
