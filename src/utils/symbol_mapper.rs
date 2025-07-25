// File: symbol_mapper.rs
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

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub name: String,
    pub country: String,
    pub yf_ticker: String,
    pub tax: i32,
}

const SYMBOL_MAPPER_FILE: &str = "data/symbol_mapper.json";
// Define the lookup table as a static HashMap loaded from symbol_mapper.json
static STOCKS_LUT: Lazy<HashMap<String, StockInfo>> = Lazy::new(|| {
    let path = Path::new(SYMBOL_MAPPER_FILE);
    let data = fs::read_to_string(path).expect("Failed to read symbol_mapper.json");
    serde_json::from_str(&data).expect("Failed to parse symbol_mapper.json")
});

pub fn extract_symbol(t212_ticker: &str) -> (String, StockInfo) {
    // 1. Use dictionary override if available
    if let Some(stock_info) = STOCKS_LUT.get(t212_ticker) {
        return (t212_ticker.to_string(), stock_info.clone());
    }

    // 2. Fallback to logic-based stripping: convert "PHNXl_EQ" -> "PHNX.L"
    let base = t212_ticker.split('_').next().unwrap_or(t212_ticker);
    let re = Regex::new(r"([A-Z]+)[a-z]$").unwrap();
    let fallback_symbol = if let Some(caps) = re.captures(base) {
        format!("{}.L", &caps[1])
    } else {
        // fallback: remove trailing lowercase letters
        let re_strip = Regex::new(r"[a-z]+$").unwrap();
        re_strip.replace(base, "").to_string()
    };

    (
        t212_ticker.to_string(),
        StockInfo {
            name: "NA".to_string(),
            yf_ticker: fallback_symbol,
            country: "NA".to_string(),
            tax: 15,
        },
    )
}
