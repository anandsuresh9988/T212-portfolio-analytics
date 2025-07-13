# T212 Portfolio Analytics

**Copyright (c) 2025 Anand Sureshkumar**

T212 Portfolio Analytics is a Rust/Axum web application for analyzing your Trading212 investment portfolio. It fetches your portfolio and dividend data from the Trading212 API, processes it, and presents interactive analytics and summaries via a modern web UI.

## Features
- Portfolio overview with profit/loss, yield, and value breakdowns
- Dividend payout history and summaries
- Monthly and per-ticker dividend analytics
- CSV export of dividend payouts

## Getting Started

### Prerequisites
- **Rust** (latest stable version recommended)
  - Install from [https://rustup.rs/](https://rustup.rs/)
- **Python 3.8+** (for stock info scripts)
  - Install from [https://python.org/](https://python.org/)
- **Trading212 account** with API access
  - Create account at [https://trading212.com/](https://trading212.com/)
  - Enable API access in your account settings

### Setup Instructions

#### 1. Clone the Repository
```bash
git clone <repository-url>
cd T212-portfolio-analytics
```

#### 2. Set Up Python Virtual Environment
The application uses Python scripts for fetching additional stock information. You need to create a virtual environment to isolate dependencies:

**On Linux/macOS:**
```bash
# Create virtual environment
python3 -m venv .venv

# Activate virtual environment
source .venv/bin/activate

# Install Python dependencies
pip install -r requirements.txt
```

**On Windows:**
```bash
# Create virtual environment
python -m venv .venv

# Activate virtual environment
.venv\Scripts\activate

# Install Python dependencies
pip install -r requirements.txt
```

**Note:** Always activate the virtual environment before running the application:
- Linux/macOS: `source .venv/bin/activate`
- Windows: `.venv\Scripts\activate`

#### 3. Configure Trading212 API Key
You need to set up your Trading212 API credentials:

1. **Get your API key:**
   - Log into your Trading212 account
   - Go to Settings → API
   - Generate a new API key
   - Copy the API key

2. **Configure the API key in the application:**
   - The API key should be configured directly in the application settings
   - Follow the application's built-in configuration process
   - Ensure your Trading212 account has API access enabled

**Important:** Keep your API key secure and never share it publicly.

#### 4. Build and Run the Application

**Build the project:**
```bash
cargo build --release
```

**Run the web application:**
```bash
cargo run
```

**Access the web interface:**
- Open your web browser
- Navigate to: [http://localhost:3001](http://localhost:3001)

### Running the Application

#### Development Mode
```bash
# Activate Python virtual environment first
source .venv/bin/activate  # Linux/macOS
# OR
.venv\Scripts\activate     # Windows

# Run in development mode
cargo run
```

#### Production Mode
```bash
# Build optimized version
cargo build --release

# Run the optimized binary
./target/release/t212-portfolio-analytics
```

#### Stopping the Application
- Press `Ctrl+C` in the terminal where the application is running

## Usage Guide

### Web Interface
1. **Portfolio Page:** View current positions, profit/loss, and portfolio value
2. **Dividends Page:** Analyze dividend history and yields
3. **Payouts Page:** Export dividend payout data as CSV

### Data Refresh
- The application automatically fetches data from Trading212 API
- Data is cached locally for performance
- Refresh manually by restarting the application

### Exporting Data
- Go to the "Payouts" page
- Click "Export CSV" to download dividend payout history
- CSV includes date, ticker, amount, and withholding tax information

## What This App Shows
- **Portfolio Overview:** Current positions, total value, and profit/loss
- **Dividend Analytics:** Yield calculations and annual income estimates
- **Payout History:** Detailed dividend payments by date and ticker
- **Tax Information:** Withholding tax (WHT) summaries
- **Performance Metrics:** Monthly and yearly dividend performance

## Troubleshooting

### Common Issues
1. **Python virtual environment not activated:**
   - Ensure you see `(.venv)` in your terminal prompt
   - Run: `source .venv/bin/activate` (Linux/macOS) or `.venv\Scripts\activate` (Windows)

2. **API key not working:**
   - Verify your API key is correct in the application settings
   - Check that your Trading212 account has API access enabled
   - Ensure you're using the correct account type (demo/live)

3. **Port already in use:**
   - The application runs on port 3001 by default
   - If port is busy, modify the port in the application configuration

4. **Build errors:**
   - Ensure Rust is installed: `rustc --version`
   - Update Rust: `rustup update`
   - Clean and rebuild: `cargo clean && cargo build`

## Disclaimer
This software is provided for personal, non-commercial, and educational use only. It interacts with your Trading212 account using your API credentials. **Use at your own risk.**

- The author is not responsible for any data loss, account issues, or financial losses.
- No warranty is provided. Always verify results independently.
- For commercial licensing or inquiries, contact: anandsureshkumar9988@gmail.com

## License
This project is licensed under a custom license:

- Free for personal and educational use
- Commercial use is strictly prohibited without written permission
- The author retains all rights to future monetization

See the [LICENSE](LICENSE) file for details.

**Author:** Anand Sureshkumar  
**Contact:** anandsureshkumar9988@gmail.com 
