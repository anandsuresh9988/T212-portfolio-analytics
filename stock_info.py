# Copyright (c) 2025 Anand Sureshkumar
# This file is part of T212 Portfolio Analytics.
# Licensed for personal and educational use only. Commercial use prohibited.
# See the LICENSE file for details.

import yfinance as yf
import sys
import json
import threading
from curl_cffi.requests.exceptions import HTTPError
from curl_cffi import requests
import traceback

symbols = sys.argv[1].split(",")
results = {}
lock = threading.Lock()
session = requests.Session(impersonate="chrome")

def fetch_info(symbol):
    try:
        ticker = yf.Ticker(symbol, session=session)
        info = ticker.info  # May raise HTTPError or others

        # Get last 4 dividends and convert Timestamp keys to string
        dividends = ticker.dividends.tail(4)
        dividends_dict = {
            date.strftime("%Y-%m-%d"): float(amount) for date, amount in dividends.items()
        }
        info["last_4_dividends"] = dividends_dict

        with lock:
            results[symbol] = info

    except HTTPError as e:
        sys.stderr.write(f"[HTTP ERROR] Symbol {symbol}: {str(e)}\n")
        traceback.print_exc(file=sys.stderr)
        with lock:
            results[symbol] = {"error": f"HTTPError: {str(e)}"}

    except Exception as e:
        sys.stderr.write(f"[ERROR] Symbol {symbol}: {str(e)}\n")
        traceback.print_exc(file=sys.stderr)
        with lock:
            results[symbol] = {"error": str(e)}

threads = [threading.Thread(target=fetch_info, args=(s,)) for s in symbols]

for t in threads:
    t.start()
for t in threads:
    t.join()

print(json.dumps(results))