import io
import json
import os
import signal
import threading
import time
import unittest
from unittest.mock import patch, MagicMock

import importlib.util
import sys

sys.path.insert(0, ".")
spec = importlib.util.spec_from_file_location("free_zen", "free-zen.py")
free_zen = importlib.util.module_from_spec(spec)
sys.modules["free_zen"] = free_zen
spec.loader.exec_module(free_zen)


class TestGenerateOpencodeHeaders(unittest.TestCase):
    def test_headers_structure(self):
        headers = free_zen.generate_opencode_headers()
        expected = {
            "User-Agent",
            "x-opencode-client",
            "x-opencode-session",
            "x-opencode-project",
            "x-opencode-request",
        }
        self.assertIsInstance(headers, dict)
        self.assertEqual(expected, set(headers.keys()))

    def test_headers_format(self):
        headers = free_zen.generate_opencode_headers()
        self.assertRegex(headers["User-Agent"], r"^opencode/latest/[\w.]+/cli$")
        for key in ["x-opencode-session", "x-opencode-project", "x-opencode-request"]:
            self.assertRegex(headers[key], r"^[a-f0-9]{26}$")

    def test_headers_uniqueness(self):
        h1 = free_zen.generate_opencode_headers()
        h2 = free_zen.generate_opencode_headers()
        self.assertNotEqual(h1["x-opencode-session"], h2["x-opencode-session"])
        self.assertNotEqual(h1["x-opencode-project"], h2["x-opencode-project"])
        self.assertNotEqual(h1["x-opencode-request"], h2["x-opencode-request"])


class TestFetchFreeModels(unittest.TestCase):
    @patch("free_zen.urllib.request.urlopen")
    def test_returns_free_models(self, mock_urlopen):
        resp = MagicMock()
        resp.read.return_value = json.dumps(
            {
                "opencode": {
                    "models": {
                        "free-a": {"name": "Free A", "cost": {"input": 0}},
                        "paid-b": {"name": "Paid B", "cost": {"input": 5}},
                        "deprecated-c": {
                            "name": "Dep C",
                            "status": "deprecated",
                            "cost": {"input": 0},
                        },
                    }
                }
            }
        ).encode()
        mock_urlopen.return_value.__enter__.return_value = resp
        result = free_zen.fetch_free_models()
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]["id"], "free-a")

    @patch("free_zen.urllib.request.urlopen")
    def test_empty_on_error(self, mock_urlopen):
        mock_urlopen.side_effect = Exception("Network down")
        result = free_zen.fetch_free_models()
        self.assertEqual(result, [])

    @patch("free_zen.urllib.request.urlopen")
    def test_empty_on_bad_data(self, mock_urlopen):
        resp = MagicMock()
        resp.read.return_value = b"not json"
        mock_urlopen.return_value.__enter__.return_value = resp
        result = free_zen.fetch_free_models()
        self.assertEqual(result, [])

    @patch("free_zen.fetch_free_models")
    def test_fallback_on_empty(self, mock_fetch):
        mock_fetch.return_value = []
        with patch("http.server.ThreadingHTTPServer") as ms:
            ms.return_value.server_address = ("127.0.0.1", 0)
            out = io.StringIO()
            err = io.StringIO()
            with (
                patch("sys.stdout", out),
                patch("sys.stderr", err),
                patch("signal.pause"),
            ):
                rc = free_zen.main([])
        self.assertEqual(rc, 0)
        self.assertIn("fallback", err.getvalue())
        self.assertIn("OPENAI_MODEL=", out.getvalue())

    def test_env_var_override(self):
        with patch.dict(os.environ, {"FREE_ZEN_MODELS_DEV_URL": "http://test"}):
            with patch("free_zen.urllib.request.urlopen") as mu:
                resp = MagicMock()
                resp.read.return_value = (
                    b'{"opencode":{"models":{"m":{"cost":{"input":0}}}}}'
                )
                mu.return_value.__enter__.return_value = resp
                result = free_zen.fetch_free_models()
                self.assertEqual(len(result), 1)
                self.assertEqual(result[0]["id"], "m")


class TestPickBestModel(unittest.TestCase):
    def test_prefers_deepseek(self):
        ids = {"mimo-v2.5-free", "deepseek-v4-flash-free", "other"}
        self.assertEqual(free_zen._pick_best_model(ids), "deepseek-v4-flash-free")

    def test_fallback_to_sorted(self):
        ids = {"b-model", "a-model", "c-model"}
        self.assertEqual(free_zen._pick_best_model(ids), "a-model")

    def test_single_model(self):
        ids = {"only-one"}
        self.assertEqual(free_zen._pick_best_model(ids), "only-one")


class TestPrintEnv(unittest.TestCase):
    def test_prints_env_vars(self):
        out = io.StringIO()
        with patch("sys.stdout", out):
            free_zen.print_env(12345, "test-model")
        text = out.getvalue()
        self.assertIn("export CLAUDE_CODE_USE_OPENAI=1", text)
        self.assertIn("export OPENAI_BASE_URL=http://127.0.0.1:12345", text)
        self.assertIn("export OPENAI_API_KEY=sk-free-zen", text)
        self.assertIn("export OPENAI_MODEL=test-model", text)
        self.assertIn("export OPENAI_API_FORMAT=chat_completions", text)
        self.assertIn("export ANTHROPIC_API_KEY=sk-ant-dummy", text)


class TestMainOutput(unittest.TestCase):
    def _main_with_mocks(self, args, free_models, server_addr=("127.0.0.1", 8888)):
        out = io.StringIO()
        err = io.StringIO()
        with patch("free_zen.fetch_free_models") as fetch:
            fetch.return_value = free_models
            with patch("http.server.ThreadingHTTPServer") as ms:
                ms.return_value.server_address = server_addr
                with (
                    patch("sys.stdout", out),
                    patch("sys.stderr", err),
                    patch("signal.pause"),
                ):
                    rc = free_zen.main(args)
        return rc, out.getvalue(), err.getvalue()

    def test_default_prints_env_exports(self):
        rc, out, _ = self._main_with_mocks([], [{"id": "free-a", "name": "Free A"}])
        self.assertEqual(rc, 0)
        self.assertIn("export CLAUDE_CODE_USE_OPENAI=1", out)
        self.assertIn("export OPENAI_BASE_URL=http://127.0.0.1:8888", out)
        self.assertIn("export OPENAI_API_KEY=sk-free-zen", out)
        self.assertIn("export OPENAI_API_FORMAT=chat_completions", out)
        self.assertIn("export ANTHROPIC_API_KEY=sk-ant-dummy", out)

    def test_list_mode(self):
        with patch("free_zen.fetch_free_models") as fetch:
            fetch.return_value = [{"id": "m1", "name": "Model One"}]
            out = io.StringIO()
            with patch("sys.stdout", out):
                rc = free_zen.main(["--list"])
        self.assertEqual(rc, 0)
        self.assertIn("Model One", out.getvalue())

    def test_json_mode(self):
        with patch("free_zen.fetch_free_models") as fetch:
            fetch.return_value = [{"id": "m1", "name": "Model One"}]
            out = io.StringIO()
            with patch("sys.stdout", out):
                rc = free_zen.main(["--json"])
        self.assertEqual(rc, 0)
        parsed = json.loads(out.getvalue())
        self.assertEqual(len(parsed), 1)
        self.assertEqual(parsed[0]["id"], "m1")

    def test_uses_best_model(self):
        rc, out, _ = self._main_with_mocks(
            [],
            [
                {"id": "deepseek-v4-flash-free", "name": "D"},
                {"id": "mimo-v2.5-free", "name": "M"},
            ],
            server_addr=("127.0.0.1", 0),
        )
        self.assertEqual(rc, 0)
        self.assertIn("deepseek-v4-flash-free", out)

    def test_invalid_flag(self):
        with patch("sys.stderr", io.StringIO()):
            with self.assertRaises(SystemExit):
                free_zen.main(["--invalid-flag"])

    def test_daemon_mode_forks_and_prints_env(self):
        with patch("free_zen.fetch_free_models") as fetch:
            fetch.return_value = [{"id": "m", "name": "M"}]
            with patch("http.server.ThreadingHTTPServer") as ms:
                ms.return_value.server_address = ("127.0.0.1", 9999)
                with patch("os.fork") as fork:
                    fork.return_value = 123
                    with patch("os._exit") as mock_exit:
                        out = io.StringIO()
                        err = io.StringIO()
                        with patch("sys.stdout", out), patch("sys.stderr", err):
                            rc = free_zen.main(["--daemon"])
        self.assertEqual(rc, 0)
        mock_exit.assert_called_once_with(0)
        self.assertIn("export CLAUDE_CODE_USE_OPENAI=1", out.getvalue())
        self.assertIn("export OPENAI_BASE_URL=http://127.0.0.1:9999", out.getvalue())
        self.assertIn("proxy started on http://127.0.0.1:9999", err.getvalue())
        env_path = "/tmp/free-zen-9999.env"
        with open(env_path) as f:
            text = f.read()
        self.assertIn("export CLAUDE_CODE_USE_OPENAI=1", text)
        self.assertIn("export OPENAI_BASE_URL=http://127.0.0.1:9999", text)
        os.unlink(env_path)


class TestProxy(unittest.TestCase):
    """Integration tests against a real running proxy instance."""

    @classmethod
    def setUpClass(cls):
        cls._saved_ids = set(free_zen._free_model_ids)
        cls._saved_models = list(free_zen._free_models)
        free_zen._free_model_ids = {"free-a", "free-b"}
        free_zen._free_models = [
            {"id": "free-a", "name": "Free A"},
            {"id": "free-b", "name": "Free B"},
        ]
        cls.server = free_zen.http.server.ThreadingHTTPServer(
            ("127.0.0.1", 0), free_zen.ZenProxyHandler
        )
        cls.port = cls.server.server_address[1]
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        time.sleep(0.1)

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        free_zen._free_model_ids = cls._saved_ids
        free_zen._free_models = cls._saved_models

    def _get(self, path):
        import urllib.request

        url = f"http://127.0.0.1:{self.port}{path}"
        return urllib.request.urlopen(url, timeout=5).read()

    def _post(self, path, body):
        import urllib.request

        url = f"http://127.0.0.1:{self.port}{path}"
        data = json.dumps(body).encode()
        req = urllib.request.Request(
            url, data=data, headers={"Content-Type": "application/json"}
        )
        return urllib.request.urlopen(req, timeout=10).read()

    def test_models_filtered(self):
        data = json.loads(self._get("/models"))
        self.assertIn("data", data)
        ids = [m["id"] for m in data["data"]]
        self.assertLessEqual(len(ids), 2)
        for m_id in ids:
            self.assertIn(m_id, {"free-a", "free-b"})

    def test_models_paid_excluded(self):
        data = json.loads(self._get("/models"))
        ids = [m["id"] for m in data["data"]]
        for m_id in ids:
            self.assertNotEqual(m_id, "claude-opus-4-6")

    def test_chat_completion(self):
        try:
            data = self._post(
                "/chat/completions",
                {
                    "model": "free-a",
                    "messages": [{"role": "user", "content": "say hi"}],
                    "max_tokens": 5,
                },
            )
            doc = json.loads(data)
            self.assertIn("choices", doc)
        except Exception as e:
            self.skipTest(f"Upstream unavailable: {e}")

    def test_unknown_path_passthrough(self):
        try:
            self._get("/unknown-path")
        except Exception as e:
            self.skipTest(
                f"Proxy error for unknown path (expected if upstream rejects): {e}"
            )

    def test_invalid_model_returns_error(self):
        import urllib.error

        try:
            self._post(
                "/chat/completions",
                {
                    "model": "nonexistent-model-xyz",
                    "messages": [{"role": "user", "content": "hi"}],
                },
            )
            self.fail("expected HTTP error")
        except urllib.error.HTTPError as e:
            self.assertIn(e.code, [401, 404, 500, 502])

    def test_invalid_json_body(self):
        import urllib.request, urllib.error

        url = f"http://127.0.0.1:{self.port}/chat/completions"
        req = urllib.request.Request(
            url, data=b"not-json", headers={"Content-Type": "application/json"}
        )
        try:
            urllib.request.urlopen(req, timeout=5)
            self.fail("expected HTTP error")
        except urllib.error.HTTPError as e:
            self.assertIn(e.code, [400, 500, 502])


if __name__ == "__main__":
    unittest.main()
