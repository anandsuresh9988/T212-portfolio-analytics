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

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendInfo {
    pub symbol: String,
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub total_investment: Decimal,
    pub annual_dividend_per_share: Decimal,
    pub annual_dividend: Decimal,
    pub dividend_yield: Decimal,
    pub yield_on_cost: Decimal,
    pub annual_wht: Decimal, // Withholding Tax
    pub annual_income_after_wht: Decimal,
    pub current_investment_val: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendMetrics {
    pub total_annual_dividend: Decimal,
    pub total_cost: Decimal,
    pub yield_on_cost: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DividendSummary {
    pub dividend_stocks: DividendMetrics,
    pub entire_portfolio: DividendMetrics,
    pub dividend_details: Vec<DividendInfo>,
}
