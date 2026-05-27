#!/bin/bash
set -euo pipefail

echo "=== Torvox 质量门 ==="
FAILED=0

echo "[1/8] cargo fmt --check"
if ! cargo fmt --check; then
  echo "FAIL: cargo fmt --check"
  FAILED=1
fi

echo "[2/8] cargo clippy --deny warnings"
if ! cargo clippy --deny warnings --all-targets --all-features; then
  echo "FAIL: cargo clippy"
  FAILED=1
fi

echo "[3/8] cargo nextest --workspace"
if ! cargo nextest run --workspace; then
  echo "FAIL: cargo nextest"
  FAILED=1
fi

echo "[4/8] proptest (10K cases)"
if ! cargo test --workspace -- proptest; then
  echo "FAIL: proptest"
  FAILED=1
fi

echo "[5/8] cargo geiger (torvox-core + torvox-terminal unsafe: 0)"
if command -v cargo-geiger &>/dev/null; then
  if ! cargo geiger --all-features 2>/dev/null | \
    grep -E "torvox-core|torvox-terminal" | \
    awk '{if ($3 != "0" || $4 != "0") {print "FAIL: unsafe found in " $1; exit 1}}'; then
    FAILED=1
  fi
else
  echo "SKIP: cargo-geiger not installed"
fi

echo "[6/8] Android lint"
if [ -d "android" ]; then
  if ! (cd android && ./gradlew lint --quiet); then
    echo "FAIL: Android lint"
    FAILED=1
  fi
else
  echo "SKIP: android/ not found"
fi

echo "[7/8] Android test"
if [ -d "android" ]; then
  if ! (cd android && ./gradlew test --quiet); then
    echo "FAIL: Android test"
    FAILED=1
  fi
else
  echo "SKIP: android/ not found"
fi

echo "[8/8] cargo audit"
if command -v cargo-audit &>/dev/null; then
  if ! cargo audit; then
    echo "WARN: cargo audit found vulnerabilities"
  fi
else
  echo "SKIP: cargo-audit not installed"
fi

echo ""
if [ $FAILED -eq 0 ]; then
  echo "=== 质量门通过 ==="
  exit 0
else
  echo "=== 质量门失败 ==="
  exit 1
fi
