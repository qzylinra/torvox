#!/usr/bin/env python3
"""Free OpenCode Zen models proxy — standalone daemon, no openclaude dependency.

Usage:
  python3 free-zen.py --daemon              # start proxy, print env, exit
  python3 free-zen.py [--port PORT]         # foreground mode
  python3 free-zen.py --list                # list free models
  python3 free-zen.py --json                # JSON output
"""

import argparse
import http.server
import io
import json
import os
import signal
import sys
import threading
import urllib.error
import urllib.request
import uuid
from typing import Dict, List, Set

MODELS_DEV_URL = "https://models.dev/api.json"
ZEN_HOST = "opencode.ai"
ZEN_PATH = "/zen/v1"

FALLBACK_MODEL_IDS = [
    "deepseek-v4-flash-free",
    "nemotron-3-ultra-free",
    "mimo-v2.5-free",
    "north-mini-code-free",
    "big-pickle",
    "hy3-free",
]

_free_model_ids: Set[str] = set()
_free_models: List[Dict[str, str]] = []


def generate_opencode_headers() -> Dict[str, str]:
    session = uuid.uuid4().hex[:26]
    project = uuid.uuid4().hex[:26]
    request = uuid.uuid4().hex[:26]
    return {
        "User-Agent": "opencode/latest/1.3.15/cli",
        "x-opencode-client": "cli",
        "x-opencode-session": session,
        "x-opencode-project": project,
        "x-opencode-request": request,
    }


class ZenProxyHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self._proxy("GET")

    def do_POST(self):
        self._proxy("POST")

    def _proxy(self, method):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length) if content_length > 0 else None
        target = ZEN_PATH + self.path
        headers = generate_opencode_headers()

        req = urllib.request.Request(
            f"https://{ZEN_HOST}{target}", data=body, headers=headers, method=method
        )

        try:
            with urllib.request.urlopen(req, timeout=600) as upstream:
                if self.path.rstrip("/").endswith("/models"):
                    data = upstream.read()
                    try:
                        doc = json.loads(data)
                        if (
                            isinstance(doc, dict)
                            and "data" in doc
                            and isinstance(doc["data"], list)
                        ):
                            doc["data"] = [
                                m for m in doc["data"] if m.get("id") in _free_model_ids
                            ]
                        data = json.dumps(doc).encode()
                    except (json.JSONDecodeError, TypeError):
                        pass
                    self._send_data(upstream.status, upstream.headers, data)
                else:
                    self._stream(upstream)
        except urllib.error.HTTPError as e:
            self._send_data(e.code, e.headers, e.read())
        except Exception as e:
            self.send_error(502, str(e))

    def _send_data(self, status, upstream_headers, body):
        self.send_response(status)
        skip = {
            "transfer-encoding",
            "content-encoding",
            "content-length",
            "connection",
            "keep-alive",
            "alt-svc",
        }
        for key, value in upstream_headers.items():
            if key.lower() not in skip:
                self.send_header(key, value)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _stream(self, upstream):
        self.send_response(upstream.status)
        skip = {
            "transfer-encoding",
            "content-encoding",
            "content-length",
            "connection",
            "keep-alive",
            "alt-svc",
        }
        for key, value in upstream.headers.items():
            if key.lower() not in skip:
                self.send_header(key, value)
        self.end_headers()
        while True:
            chunk = upstream.read(8192)
            if not chunk:
                break
            self.wfile.write(chunk)
            self.wfile.flush()

    def log_message(self, format, *args):
        pass


def fetch_free_models(timeout: int = 10) -> List[Dict[str, str]]:
    url = os.environ.get("FREE_ZEN_MODELS_DEV_URL") or MODELS_DEV_URL
    req = urllib.request.Request(url, headers=generate_opencode_headers(), method="GET")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read())
    except Exception as e:
        print(f"free-zen: warning: models.dev fetch failed: {e}", file=sys.stderr)
        return []

    models_data = data.get("opencode", {}).get("models", {})
    if not isinstance(models_data, dict):
        print("free-zen: warning: unexpected models.dev structure", file=sys.stderr)
        return []

    result = []
    for model_id, info in models_data.items():
        if not isinstance(info, dict):
            continue
        if info.get("status") == "deprecated":
            continue
        cost = info.get("cost")
        if cost is None or not isinstance(cost, dict):
            continue
        if cost.get("input") != 0:
            continue

        result.append({"id": model_id, "name": info.get("name", model_id)})

    result.sort(key=lambda m: m["name"].lower())
    return result


def _pick_best_model(ids: Set[str]) -> str:
    return (
        "deepseek-v4-flash-free" if "deepseek-v4-flash-free" in ids else sorted(ids)[0]
    )


def _env_text(port: int, model: str) -> str:
    return (
        f"# free-zen proxy env — source this or set manually\n"
        f"# Proxy: http://127.0.0.1:{port}\n"
        f"# Model: {model}\n"
        f"CLAUDE_CODE_USE_OPENAI=1\n"
        f"OPENAI_BASE_URL=http://127.0.0.1:{port}\n"
        f"OPENAI_API_KEY=\n"
        f"OPENAI_MODEL={model}\n"
        f"OPENAI_API_FORMAT=chat_completions\n"
    )


def print_env(port: int, model: str) -> None:
    print(_env_text(port, model), end="")


def start_server(port: int) -> http.server.ThreadingHTTPServer:
    return http.server.ThreadingHTTPServer(("127.0.0.1", port), ZenProxyHandler)


def run_foreground(server: http.server.ThreadingHTTPServer) -> None:
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    print(f"free-zen: proxy listening on http://127.0.0.1:{port}", file=sys.stderr)
    print_env(port, _pick_best_model(_free_model_ids))
    try:
        signal.pause()
    except KeyboardInterrupt:
        pass
    server.shutdown()


def run_daemon(server: http.server.ThreadingHTTPServer) -> None:
    port = server.server_address[1]
    env_path = f"/tmp/free-zen-{port}.env"
    pid = os.fork()
    if pid > 0:
        with open(env_path, "w") as f:
            f.write(_env_text(port, _pick_best_model(_free_model_ids)))
        print(f"free-zen: proxy started on http://127.0.0.1:{port}", file=sys.stderr)
        print(f"free-zen: source {env_path} to set env vars", file=sys.stderr)
        os._exit(0)
    os.setsid()
    try:
        devnull = os.open(os.devnull, os.O_RDWR)
        os.dup2(devnull, sys.stdout.fileno())
        os.dup2(devnull, sys.stderr.fileno())
        os.close(devnull)
    except (OSError, io.UnsupportedOperation):
        pass
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        signal.pause()
    except KeyboardInterrupt:
        pass
    server.shutdown()


def main(argv: List[str]) -> int:
    global _free_models, _free_model_ids

    parser = argparse.ArgumentParser(
        description="Free OpenCode Zen models proxy — standalone daemon, no openclaude dependency."
    )
    parser.add_argument("--list", action="store_true", help="List free models and exit")
    parser.add_argument("--json", action="store_true", help="JSON output")
    parser.add_argument(
        "--daemon", action="store_true", help="Fork to background, print env to stdout"
    )
    parser.add_argument(
        "--port", type=int, default=0, help="Proxy port (default: random)"
    )
    parser.add_argument(
        "--timeout", type=int, default=10, help="models.dev fetch timeout"
    )
    parser.add_argument(
        "--probe", action="store_true", help="Verify free models against zen /v1/models"
    )
    args = parser.parse_args(argv)

    _free_models = fetch_free_models(args.timeout)
    _free_model_ids = {m["id"] for m in _free_models}

    if not _free_models:
        print(
            "free-zen: warning: using hardcoded fallback model list (remote unavailable)",
            file=sys.stderr,
        )
        _free_model_ids = set(FALLBACK_MODEL_IDS)
        _free_models = [{"id": m, "name": m} for m in FALLBACK_MODEL_IDS]

    if args.list:
        for m in _free_models:
            print(m["name"])
        return 0

    if args.json:
        print(json.dumps(_free_models, indent=2))
        return 0

    if args.probe:
        try:
            req = urllib.request.Request(
                f"https://opencode.ai/zen/v1/models",
                headers=generate_opencode_headers(),
            )
            with urllib.request.urlopen(req, timeout=args.timeout) as r:
                zen_data = json.loads(r.read())
                zen_ids = {
                    m["id"] for m in zen_data.get("data", []) if isinstance(m, dict)
                }
            missing = _free_model_ids - zen_ids
            if missing:
                print(
                    f"free-zen: warning: models not in zen /v1/models: {missing}",
                    file=sys.stderr,
                )
        except Exception as e:
            print(f"free-zen: warning: zen probe failed: {e}", file=sys.stderr)
        return 0

    server = start_server(args.port)
    port = server.server_address[1]

    if args.daemon:
        run_daemon(server)
    else:
        run_foreground(server)

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
