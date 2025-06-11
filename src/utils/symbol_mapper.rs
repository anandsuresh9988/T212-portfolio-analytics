// File: symbol_mapper.rs
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

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub name: String,
    pub country: String,
    pub yf_ticker: String,
    pub tax: i32,
}

// Define the lookup table as a static HashMap
static STOCKS_LUT: Lazy<HashMap<String, StockInfo>> = Lazy::new(|| {
    let mut lut = HashMap::new();

    lut.insert(
        "AAPL".into(),
        StockInfo {
            name: "Apple Inc.".into(),
            country: "US".into(),
            yf_ticker: "AAPL".into(),
            tax: 15,
        },
    );
    lut.insert(
        "TSLA_US_EQ".into(),
        StockInfo {
            name: "Tesla".into(),
            country: "US".into(),
            yf_ticker: "TSLA".into(),
            tax: 15,
        },
    );
    lut.insert(
        "LGENl_EQ".into(),
        StockInfo {
            name: "LGEN.L".into(),
            country: "UK".into(),
            yf_ticker: "LGEN.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "LLOYl_EQ".into(),
        StockInfo {
            name: "LLoyds Bank.".into(),
            country: "UK".into(),
            yf_ticker: "LLOY.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "VHYLl_EQ".into(),
        StockInfo {
            name: "Fund.".into(),
            country: "UK".into(),
            yf_ticker: "VHYL.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "IUKDl_EQ".into(),
        StockInfo {
            name: "TFund.".into(),
            country: "UK".into(),
            yf_ticker: "IUKD.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "GRGl_EQ".into(),
        StockInfo {
            name: "Gregs UK.".into(),
            country: "UK".into(),
            yf_ticker: "GRG.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "HSBAl_EQ".into(),
        StockInfo {
            name: "HSBC UK.".into(),
            country: "NA".into(),
            yf_ticker: "HSBA.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "EHYGl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "NA".into(),
            yf_ticker: "EHYG.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "VUKGl_EQ".into(),
        StockInfo {
            name: "Fund.".into(),
            country: "NA".into(),
            yf_ticker: "VUKG.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BTl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "NA".into(),
            yf_ticker: "BT-A.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "FB_US_EQ".into(),
        StockInfo {
            name: "Meta.".into(),
            country: "US".into(),
            yf_ticker: "META".into(),
            tax: 15,
        },
    );
    lut.insert(
        "BATSl_EQ".into(),
        StockInfo {
            name: "BATS.".into(),
            country: "UK".into(),
            yf_ticker: "BATS.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BARCl_EQ".into(),
        StockInfo {
            name: "Barclays.".into(),
            country: "UK".into(),
            yf_ticker: "BARC.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "ISFl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "UK".into(),
            yf_ticker: "ISF.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "IMBl_EQ".into(),
        StockInfo {
            name: "Imperial Brands.".into(),
            country: "UK".into(),
            yf_ticker: "IMB.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "IINDl_EQ".into(),
        StockInfo {
            name: "Fund.".into(),
            country: "UK".into(),
            yf_ticker: "IIND.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "ICDUl_EQ".into(),
        StockInfo {
            name: "Test Inc.".into(),
            country: "NA".into(),
            yf_ticker: "ICDU.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "EQGBl_EQ".into(),
        StockInfo {
            name: "EQGB".into(),
            country: "NA".into(),
            yf_ticker: "EQGB.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BRK_B_US_EQ".into(),
        StockInfo {
            name: "Berkshare".into(),
            country: "USA".into(),
            yf_ticker: "BRK-B".into(),
            tax: 15,
        },
    );
    lut.insert(
        "RRl_EQ".into(),
        StockInfo {
            name: "Rolls Royce.".into(),
            country: "UK".into(),
            yf_ticker: "RR.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "SSLNl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "UK".into(),
            yf_ticker: "SSLN.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "SHELl_EQ".into(),
        StockInfo {
            name: "SHELL.".into(),
            country: "UK".into(),
            yf_ticker: "SHEL.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BPl_EQ".into(),
        StockInfo {
            name: "BP_UK.".into(),
            country: "UK".into(),
            yf_ticker: "BP.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "FRINl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "UK".into(),
            yf_ticker: "FRIN.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "CARDl_EQ".into(),
        StockInfo {
            name: "Card factory".into(),
            country: "UK".into(),
            yf_ticker: "CARD.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BAl_EQ".into(),
        StockInfo {
            name: "NA.".into(),
            country: "UK".into(),
            yf_ticker: "BA.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "SGLNl_EQ".into(),
        StockInfo {
            name: "Fund.".into(),
            country: "UK".into(),
            yf_ticker: "SGLN.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "VUAGl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "UK".into(),
            yf_ticker: "VUAG.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "RIOl_EQ".into(),
        StockInfo {
            name: "".into(),
            country: "UK".into(),
            yf_ticker: "RIO.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "BABl_EQ".into(),
        StockInfo {
            name: "".into(),
            country: "uk".into(),
            yf_ticker: "BAB.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "NGl_EQ".into(),
        StockInfo {
            name: ".".into(),
            country: "NA".into(),
            yf_ticker: "NG.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "VWRPl_EQ".into(),
        StockInfo {
            name: "Fund".into(),
            country: "UK".into(),
            yf_ticker: "VWRP.L".into(),
            tax: 0,
        },
    );
    lut.insert(
        "VACQ_US_EQ".into(),
        StockInfo {
            name: "Rocket labs".into(),
            country: "US".into(),
            yf_ticker: "RKLB".into(),
            tax: 15,
        },
    );

    lut.insert(
        "TEST".into(),
        StockInfo {
            name: "Test Inc.".into(),
            country: "NA".into(),
            yf_ticker: "TEST".into(),
            tax: 15,
        },
    );

    // Add other enties here

    lut
});

pub fn extract_symbol(t212_ticker: &str) -> (String, StockInfo) {
    // 1. Use dictionary override if available
    if let Some(stock_info) = STOCKS_LUT.get(t212_ticker) {
        return (t212_ticker.to_string(), stock_info.clone());
    }

    // 2. Fallback to logic-based stripping: remove trailing lowercase letters
    let base = t212_ticker.split('_').next().unwrap_or(t212_ticker);
    let re = Regex::new(r"[a-z]+$").unwrap();
    let fallback_symbol = re.replace(base, "").to_string();

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
