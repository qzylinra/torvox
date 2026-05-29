#!/bin/bash
set -euo pipefail

# 设置 Java 25 (Android 构建需要)
if [ -d "/usr/lib/jvm/temurin-25-jdk-amd64" ]; then
	export JAVA_HOME=/usr/lib/jvm/temurin-25-jdk-amd64
fi

echo "=== Torvox 质量门 ==="
echo "Java: $(java -version 2>&1 | head -1)"
FAILED=0

echo "[1/8] cargo fmt --check"
if ! cargo fmt --check; then
	echo "FAIL: cargo fmt --check"
	FAILED=1
fi

echo "[2/8] cargo clippy -- -D warnings"
if ! cargo clippy --workspace --all-targets -- -D warnings; then
	echo "FAIL: cargo clippy"
	FAILED=1
fi

echo "[3/8] cargo test --workspace"
if ! cargo test --workspace; then
	echo "FAIL: cargo test"
	FAILED=1
fi

echo "[4/8] proptest"
if ! cargo test --workspace -- proptest; then
	echo "FAIL: proptest"
	FAILED=1
fi

echo "[5/8] cargo geiger"
if command -v cargo-geiger &>/dev/null; then
	if ! cargo geiger --all-features 2>/dev/null |
		grep -E "torvox-core|torvox-terminal" |
		awk '{if ($3 != "0" || $4 != "0") {print "FAIL: unsafe found in " $1; exit 1}}'; then
		FAILED=1
	fi
else
	echo "SKIP: cargo-geiger not installed"
fi

echo "[6/8] Android lint"
if [ -d "android" ] && [ -n "${JAVA_HOME:-}" ]; then
	if ! (cd android && ./gradlew lint --quiet); then
		echo "FAIL: Android lint"
		FAILED=1
	fi
else
	echo "SKIP: Android lint (no JAVA_HOME)"
fi

echo "[7/8] Android test"
if [ -d "android" ] && [ -n "${JAVA_HOME:-}" ]; then
	if ! (cd android && ./gradlew test --quiet); then
		echo "FAIL: Android test"
		FAILED=1
	fi
else
	echo "SKIP: Android test (no JAVA_HOME)"
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
