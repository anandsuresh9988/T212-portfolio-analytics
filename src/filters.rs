// Copyright (c) 2025 Anand Sureshkumar
// This file is part of T212 Portfolio Analytics.
// Licensed for personal and educational use only. Commercial use prohibited.
// See the LICENSE file for details.

use std::fmt::Write;

pub fn format(value: &str, fmt: &str) -> askama::Result<String> {
    let num: f64 = value.parse().unwrap_or(0.0);
    let mut result = String::new();
    write!(&mut result, "{:.*}", fmt.parse::<usize>().unwrap_or(2), num)?;
    Ok(result)
}

pub fn float(value: &str) -> askama::Result<f64> {
    Ok(value.parse().unwrap_or(0.0))
} 