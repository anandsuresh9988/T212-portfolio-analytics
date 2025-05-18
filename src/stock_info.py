import yfinance as yf
import sys
import json
import threading
from curl_cffi import requests

import traceback

symbols = sys.argv[1].split(",")
results = {}
lock = threading.Lock()
session = requests.Session(impersonate="chrome")

def fetch_info(symbol):
    try:
        ticker = yf.Ticker(symbol, session=session)
        info = ticker.info
        with lock:
            results[symbol] = info
    except Exception as e:
        # Print detailed error to stderr for logs/debugging
        sys.stderr.write(f"[ERROR] Symbol {symbol}: {str(e)}\n")
        traceback.print_exc(file=sys.stderr)
        with lock:
            results[symbol] = {"error": str(e)}

threads = [threading.Thread(target=fetch_info, args=(s,)) for s in symbols]
[t.start() for t in threads]
[t.join() for t in threads]

print(json.dumps(results))
