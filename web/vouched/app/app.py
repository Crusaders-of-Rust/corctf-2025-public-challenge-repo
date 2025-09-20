#!/usr/bin/env python3
from flask import Flask, render_template, request
import hashlib, hmac, binascii
import random
try:
    from secret import FLAG
except ImportError:
    FLAG = "FLAG{EXAMPLE_FLAG}"

app = Flask(__name__)

PBKDF2_ITERATIONS = 1750000
DKLEN = 32


def generate_voucher() -> str:
    length = 12
    dash_positions = [1, 4, 8]

    chars = []
    for i in range(length):
        if i in dash_positions:
            chars.append('-')
        else:
            chars.append(random.choice("ABCDEF0123456789"))

    return ''.join(chars)


VOUCHER_CODE = generate_voucher()
print(f"[DEBUG] Voucher code: {VOUCHER_CODE}")


def calculate_signature(password: str, salt: str) -> str:
    return binascii.hexlify(
        hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), PBKDF2_ITERATIONS, dklen=DKLEN)
    ).decode()


@app.route("/")
def index():
    return render_template("index.html")


@app.route("/check", methods=["POST"])
def check():
    j = request.get_json(force=True)
    voucher = j.get("voucher", "")
    signature = j.get("signature", "")

    if len(voucher) != len(VOUCHER_CODE):
        return "Code incorrect"

    for i, ch in enumerate(voucher):
        
        if VOUCHER_CODE[i] != ch:
            return "Code incorrect"
        
        # Tampering protection
        ua = request.headers.get("User-Agent", "")
        expected = calculate_signature(voucher, ua)
        if not hmac.compare_digest(signature, expected):
            return "Tampering detected"
        
    return f"See you at corCTF 2026, your ticket is: {FLAG}"


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000, debug=False)
