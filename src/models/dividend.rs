/*
 * Copyright (c) 2024 Anand S <anandsuresh9988@gmail.com>
 *
 * This file is part of the Portfolio Management project.
 */
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
