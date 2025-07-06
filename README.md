# T212 Portfolio Analytics

**Copyright (c) 2025 Anand Sureshkumar**

T212 Portfolio Analytics is a Rust/Axum web application for analyzing your Trading212 investment portfolio. It fetches your portfolio and dividend data from the Trading212 API, processes it, and presents interactive analytics and summaries via a modern web UI.

## Features
- Portfolio overview with profit/loss, yield, and value breakdowns
- Dividend payout history and summaries
- Monthly and per-ticker dividend analytics
- CSV export of dividend payouts
- Modern, responsive web UI (Bootstrap)

## Architecture

```mermaid
flowchart TD
    User[User (Web Browser)]
    UI[Web UI (Axum + Askama Templates)]
    WebServer[Axum Web Server]
    Portfolio[Portfolio Logic & Models]
    Trading212[Trading212 API]
    Python[Python Stock Info Script]
    Cache[Cache/JSON Files]

    User <--> UI
    UI <--> WebServer
    WebServer <--> Portfolio
    Portfolio <--> Trading212
    Portfolio <--> Python
    Portfolio <--> Cache
    Trading212 <--> WebServer
```

- **User** interacts with the app via a web browser.
- **Web UI** is rendered using Askama templates and Bootstrap.
- **Axum Web Server** handles HTTP requests and routes.
- **Portfolio Logic** fetches and processes data from Trading212 and Python scripts, and manages caching.
- **Trading212 API** provides portfolio and dividend data.
- **Python Script** fetches additional stock info (e.g., from Yahoo Finance).
- **Cache** stores intermediate data for performance.

## Getting Started

### Prerequisites
- Rust (latest stable recommended)
- Trading212 account with API access
- Python 3 (for some stock info scripts)

### Setup
1. **Clone the repository:**
   ```sh
   git clone <repo-url>
   cd T212-portfolio-analytics
   ```
2. **Set up Python virtual environment:**
   ```sh
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```
   (Replace `python3` with `python` if necessary on your system)
3. **Set your Trading212 API key:**
   - Create a `.env` file in the project root:
     ```sh
     echo "TRADING212_API_TOKEN=your_api_key_here" > .env
     ```
   - (Optional) Set `T212_TARGET=live` or `T212_TARGET=demo` in `.env`.
4. **Build the project:**
   ```sh
   cargo build
   ```
5. **Run the application:**
   ```sh
   cargo run
   ```
6. **Open your browser:**
   - Go to [http://localhost:3001](http://localhost:3001)

## Usage
- The app will periodically fetch and update your portfolio and dividend data.
- Navigate between Portfolio, Dividends, and Payouts using the navbar.
- Export your dividend payout history as CSV from the Payouts page.

## What This App Shows
- Current portfolio positions, value, and profit/loss
- Dividend yield and annual income estimates
- Detailed dividend payout history (by date, ticker, and month)
- Withholding tax (WHT) summaries

## Disclaimer
This software is provided for personal, non-commercial, and educational use only. It interacts with your Trading212 account using your API credentials. **Use at your own risk.**

- The author is not responsible for any data loss, account issues, or financial losses.
- No warranty is provided. Always verify results independently.
- For commercial licensing or inquiries, contact: anandsuresh9988@gmail.com

## License
This project is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License. See the [LICENSE](LICENSE) file for full terms. 
