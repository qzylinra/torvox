#!/usr/bin/env python3
"""Free OpenCode Zen models for openclaude via local HTTP proxy."""

import argparse
import http.server
import json
import os
import signal
import subprocess
import sys
import threading
import urllib.error
import urllib.request
import uuid
from typing import Dict, List, Optional, Set

MODELS_DEV_URL = "https://models.dev/api.json"
ZEN_BASE_URL = "https://opencode.ai/zen/v1"

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

        target = ZEN_BASE_URL + self.path
        headers = generate_opencode_headers()

        req = urllib.request.Request(target, data=body, headers=headers, method=method)

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
                    self._send_regular(upstream.status, upstream.headers, data)
                else:
                    self._stream(upstream)
        except urllib.error.HTTPError as e:
            self._send_regular(e.code, e.headers, e.read())
        except Exception as e:
            self.send_error(502, str(e))

    def _send_regular(self, status, upstream_headers, body):
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


def main(argv: List[str]) -> int:
    global _free_models, _free_model_ids

    parser = argparse.ArgumentParser(
        description="Free OpenCode Zen models for openclaude via local proxy.\n"
        "Starts an HTTP proxy on localhost that injects opencode-style headers\n"
        "and filters /v1/models to only show free models.",
    )
    parser.add_argument("--list", action="store_true", help="List free models and exit")
    parser.add_argument(
        "--json", action="store_true", help="Output free models as JSON and exit"
    )
    parser.add_argument(
        "--probe", action="store_true", help="Probe zen /v1/models for verification"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=0,
        help="Local proxy port (default: random available)",
    )
    parser.add_argument(
        "--launch", action="store_true", help="Launch openclaude through the proxy"
    )
    parser.add_argument(
        "--launch-cmd",
        default="openclaude",
        help="Command to launch (default: openclaude)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=10,
        help="Timeout for models.dev fetch (default: 10)",
    )
    args = parser.parse_args(argv)

    _free_models = fetch_free_models(args.timeout)
    _free_model_ids = {m["id"] for m in _free_models}

    if not _free_models:
        print(
            "free-zen: warning: using hardcoded fallback model list (no free models from remote)",
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
                "https://opencode.ai/zen/v1/models",
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
            print(
                f"free-zen: warning: zen probe failed: {e}",
                file=sys.stderr,
            )

    server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), ZenProxyHandler)
    port = server.server_address[1]
    proxy_thread = threading.Thread(target=server.serve_forever, daemon=True)
    proxy_thread.start()

    best = (
        "deepseek-v4-flash-free"
        if "deepseek-v4-flash-free" in _free_model_ids
        else sorted(_free_model_ids)[0]
    )

    lines = []
    lines.append("# Generated by free-zen.py - free OpenCode Zen models")
    lines.append(f"# Proxy: http://127.0.0.1:{port} -> {ZEN_BASE_URL}")
    lines.append(f"# Free model: {best}")
    lines.append("# Usage: source <(python free-zen.py) && openclaude")
    lines.append("export CLAUDE_CODE_USE_OPENAI=1")
    lines.append(f"export OPENAI_BASE_URL=http://127.0.0.1:{port}")
    lines.append("export OPENAI_API_KEY=")
    lines.append(f"export OPENAI_MODEL={best}")
    lines.append("export OPENAI_API_FORMAT=chat_completions")
    print("\n".join(lines))
    sys.stdout.flush()

    if args.launch:
        env = os.environ.copy()
        env.update(
            {
                "CLAUDE_CODE_USE_OPENAI": "1",
                "OPENAI_BASE_URL": f"http://127.0.0.1:{port}",
                "OPENAI_API_KEY": "",
                "OPENAI_MODEL": best,
                "OPENAI_API_FORMAT": "chat_completions",
            }
        )
        try:
            proc = subprocess.Popen([args.launch_cmd], env=env)
            proc.wait()
        except FileNotFoundError:
            print(
                f"free-zen: error: '{args.launch_cmd}' not found on PATH",
                file=sys.stderr,
            )
            server.shutdown()
            return 2
        finally:
            server.shutdown()
    else:
        print(
            f"free-zen: proxy on http://127.0.0.1:{port} - press Ctrl+C to stop",
            file=sys.stderr,
        )
        try:
            signal.pause()
        except KeyboardInterrupt:
            pass
        server.shutdown()

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
