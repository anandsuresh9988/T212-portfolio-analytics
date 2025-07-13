# T212 Portfolio Analytics

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.8%2B-blue?logo=python)](https://www.python.org/)
[![License: Custom Non-Commercial](https://img.shields.io/badge/license-Custom%20Non--Commercial-red)](LICENSE)

**An app to fetch Trading 212 investment portfolio and predict the dividends and yield**

---

## ✨ Features

- 📊 **Portfolio Dashboard:** See your positions, profit/loss, and value breakdowns
- 💸 **Dividend Analytics:** Track payouts, yields, and monthly/yearly summaries
- 📅 **Upcoming Payments:** Predict and visualize future dividends
- 🗃️ **CSV Export:** Download your dividend history for your records

---

## 🚀 Quick Start

1. **Clone the repo**
   ```bash
   git clone <repository-url>
   cd T212-portfolio-analytics
   ```

2. **Set up Python environment**
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

3. **Build and run**
   ```bash
   cargo build --release
   cargo run
   ```

4. **Open your browser:**  
   [http://127.0.0.1:3000/dividends](http://127.0.0.1:3000/dividends)

---

## ⚙️ Requirements

- [Rust](https://rustup.rs/) (latest stable)
- [Python 3.8+](https://www.python.org/) (for stock info scripts)
- Trading 212 account with API access

---

## 🔑 Setup: Trading 212 API

- Log in to Trading 212, go to **Settings → API**, and generate an API key.
- Enter your API key in the app’s settings page after first launch.

---

## 🖥️ Usage

- **Portfolio:** View your current holdings, values, and P/L
- **Dividends:** Shows dividends of each stock for the year
- **Payouts:** Shows the dividends received so far
- **Settings:** Configure API key, currency, and update intervals

---

## 🛡️ Disclaimer

- This app uses your Trading 212 API credentials. **Use at your own risk.**
- The author is not responsible for any data loss, account issues, or financial losses.
- No warranty is provided. Always verify results independently.

---

## 📄 License

**Custom License:**

- Free for personal and educational use
- **Commercial use is strictly prohibited** without written permission
- The author retains all rights to future monetization

See the [LICENSE](LICENSE) file for details.

---

**Author:** Anand Sureshkumar  
**Contact:** anandsuresh9988@gmail.com

---

> _Want to use this for commercial purposes? Contact the author for licensing!_ 
