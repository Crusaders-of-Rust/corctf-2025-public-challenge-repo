from __future__ import annotations
from flask import Flask, abort, send_file, Response, request
from pathlib import Path

import uuid

from secret import API_TOKEN

fs_app = Flask(__name__)

UPLOAD_DIR = (Path(__file__).parent / "uploads").resolve()

@fs_app.after_request
def security_headers(resp: Response):
    resp.headers.setdefault("X-Content-Type-Options", "nosniff")
    resp.headers.setdefault("Cache-Control", "private, max-age=60")
    return resp

@fs_app.route("/<uuid_str>", methods=["GET"])
def fetch_by_uuid(uuid_str: str):
    cookie = request.headers.get("Authorization")
    if cookie.split(" ")[1] != API_TOKEN:
        abort(401)
    try:
        uuid.UUID(uuid_str)
    except ValueError:
        abort(404)

    for path in sorted(UPLOAD_DIR.glob(f"{uuid_str}.png")):
        if path.is_file():
            return send_file(path)

    abort(404)