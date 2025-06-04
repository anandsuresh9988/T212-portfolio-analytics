// File: dividend.rs
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendInfo {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub total_investment: f64,
    pub annual_dividend_per_share: f64,
    pub annual_dividend: f64,
    pub dividend_yield: f64,
    pub yield_on_cost: f64,
    pub annual_wht: f64, // Withholding Tax
    pub annual_income_after_wht: f64,
    pub current_investment_val: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendMetrics {
    pub total_annual_dividend: f64,
    pub total_cost: f64,
    pub yield_on_cost: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendSummary {
    pub dividend_stocks: DividendMetrics,
    pub entire_portfolio: DividendMetrics,
    pub dividend_details: Vec<DividendInfo>,
    pub total_annual_dividend: f64,
    pub total_cost: f64,
    pub yield_on_cost: f64,
}
