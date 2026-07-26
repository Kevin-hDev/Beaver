#!/usr/bin/env python3
import argparse
import hmac
import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer

from forecast_runtime.adapters import load_adapter
from forecast_runtime.limits import MAX_PAYLOAD_BYTES, MAX_RESPONSE_BYTES
from forecast_runtime.validation import validate_horizon, validate_quantiles

ADAPTER = None
MODEL_NAME = ""
FAMILY_ID = ""
AUTH_TOKEN = ""


class ForecastHandler(BaseHTTPRequestHandler):
    server_version = "BeaverForecastSidecar/3.0"

    def do_GET(self):
        if self.path != "/health":
            self._send_json(404, {"error": "not_found"})
            return
        if not self._is_authorized():
            self._send_json(401, {"error": "unauthorized"})
            return
        self._send_json(200, {"ok": True, "model": MODEL_NAME, "family": FAMILY_ID})

    def do_POST(self):
        if self.path != "/predict":
            self._send_json(404, {"error": "not_found"})
            return
        if not self._is_authorized():
            self._send_json(401, {"error": "unauthorized"})
            return
        try:
            payload = self._read_payload()
            horizon = validate_horizon(payload.get("horizon"))
            quantiles = validate_quantiles(payload.get("quantiles"))
            result = ADAPTER.predict(payload, horizon, quantiles)
            self._send_json(200, result)
        except ValueError as exc:
            self._send_json(400, {"error": str(exc)})
        except Exception:
            self._send_json(500, {"error": "prediction_failed"})

    def log_message(self, _format, *_args):
        return

    def _read_payload(self):
        length = int(self.headers.get("content-length", "0"))
        if length <= 0 or length > MAX_PAYLOAD_BYTES:
            raise ValueError("invalid_payload")
        payload = json.loads(self.rfile.read(length))
        if not isinstance(payload, dict):
            raise ValueError("invalid_payload")
        return payload

    def _is_authorized(self):
        provided = self.headers.get("x-clgo-forecast-token", "")
        return bool(AUTH_TOKEN) and hmac.compare_digest(provided, AUTH_TOKEN)

    def _send_json(self, status, payload):
        body = json.dumps(payload).encode("utf-8")
        if len(body) > MAX_RESPONSE_BYTES:
            status = 500
            body = b'{"error":"response_too_large"}'
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main():
    global ADAPTER
    global MODEL_NAME
    global FAMILY_ID
    global AUTH_TOKEN

    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--family", required=True)
    parser.add_argument("--models-dir", required=True)
    args = parser.parse_args()

    AUTH_TOKEN = os.environ.get("CLGO_FORECAST_TOKEN", "")
    if not AUTH_TOKEN:
        raise SystemExit("missing_auth_token")

    MODEL_NAME = args.model
    FAMILY_ID = args.family
    device = os.environ.get("CLGO_FORECAST_DEVICE", "gpu")
    ADAPTER = load_adapter(args.family, args.model, args.models_dir, device)

    server = HTTPServer(("127.0.0.1", args.port), ForecastHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()


if __name__ == "__main__":
    main()
