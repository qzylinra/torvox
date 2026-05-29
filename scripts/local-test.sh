#!/bin/bash
set -euo pipefail

# Torvox 快速本地测试 (无需模拟器)
# 使用方法: ./scripts/local-test.sh
#
# 测试内容:
#   1. clippy + fmt
#   2. Rust 全部测试
#   3. Android lint + unit tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/test-results"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[OK]${NC} $1"; }
fail() {
	echo -e "${RED}[FAIL]${NC} $1"
	exit 1
}

mkdir -p "$RESULTS_DIR"
cd "$PROJECT_DIR"

echo "=== Torvox 本地测试 ==="
echo ""

# 1. Clippy
echo "[1/5] cargo clippy"
cargo clippy --workspace -- -D warnings 2>&1 | tee "$RESULTS_DIR/clippy.log" >/dev/null
log "clippy 零警告"

# 2. Fmt
echo "[2/5] cargo fmt"
cargo fmt --check 2>&1 | tee "$RESULTS_DIR/fmt.log" >/dev/null
log "格式化通过"

# 3. Rust 测试
echo "[3/5] cargo test"
cargo test --workspace 2>&1 | tee "$RESULTS_DIR/test.log"
TOTAL=$(cargo test --workspace 2>&1 | grep "test result:" | awk -F'[; ]' '{for(i=1;i<=NF;i++) if($i=="passed") sum+=$(i-1)} END{print sum}')
log "Rust 测试通过: $TOTAL 个"

# 4. Android lint
echo "[4/5] Android lint"
cd android && ./gradlew lint --quiet 2>&1 | tee "$RESULTS_DIR/android-lint.log" >/dev/null
cd ..
log "Android lint 通过"

# 5. Android unit tests
echo "[5/5] Android unit tests"
cd android && ./gradlew test --quiet 2>&1 | tee "$RESULTS_DIR/android-test.log" >/dev/null
cd ..
log "Android 单元测试通过"

echo ""
echo "=== 全部通过 ==="
echo "结果: $RESULTS_DIR/"
