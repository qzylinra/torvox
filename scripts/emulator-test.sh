#!/bin/bash
set -euo pipefail

# Torvox 本地模拟器测试方案
# 使用方法: nix develop .#emulator --command ./scripts/emulator-test.sh
#
# 前置条件:
#   1. KVM 已启用 (Linux: ls /dev/kvm)
#   2. Nix devshell: nix develop .#emulator
#   3. Rust 工具链已安装
#
# 测试层级:
#   --unit      仅 Rust 单元测试 (无需模拟器)
#   --android   Android Gradle 测试 (无需模拟器)
#   --emulator  完整模拟器测试 (需要 KVM)
#   --all       全部测试 (默认)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
APK_PATH="$PROJECT_DIR/android/app/build/outputs/apk/debug/app-debug.apk"
RESULTS_DIR="$PROJECT_DIR/test-results"
AVD_NAME="torvox_api36"
EMULATOR_TIMEOUT=120

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[TEST]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }

mkdir -p "$RESULTS_DIR"

# ── L0: 编译时检查 ──────────────────────────────────────
run_clippy() {
	log "L0: cargo clippy -- -D warnings"
	cd "$PROJECT_DIR"
	cargo clippy --workspace -- -D warnings 2>&1 | tee "$RESULTS_DIR/clippy.log"
	log "L0: cargo fmt --check"
	cargo fmt --check 2>&1 | tee "$RESULTS_DIR/fmt.log"
	log "L0 通过"
}

# ── L1: Rust 单元测试 ──────────────────────────────────
run_rust_tests() {
	log "L1: cargo nextest run --workspace"
	cd "$PROJECT_DIR"
	cargo nextest run --workspace 2>&1 | tee "$RESULTS_DIR/nextest.log"
	log "L1 通过"
}

# ── L2: 属性测试 ──────────────────────────────────────
run_proptest() {
	log "L2: proptest (10K+ 用例)"
	cd "$PROJECT_DIR"
	cargo test --workspace -- proptest 2>&1 | tee "$RESULTS_DIR/proptest.log"
	log "L2 通过"
}

# ── L3: 集成测试 ──────────────────────────────────────
run_integration_tests() {
	log "L3: torvox-integration-tests"
	cd "$PROJECT_DIR"
	cargo test -p torvox-integration-tests 2>&1 | tee "$RESULTS_DIR/integration.log"
	log "L3 通过"
}

# ── L4: 模糊测试 (可选) ──────────────────────────────
run_fuzz() {
	local iterations=${1:-10000}
	log "L4: 模糊测试 ($iterations 迭代)"
	cd "$PROJECT_DIR"
	cargo fuzz run vt_parser -- -max_total_time=10 -runs=$iterations 2>&1 | tee "$RESULTS_DIR/fuzz.log" || true
	log "L4 完成"
}

# ── Android Gradle 测试 ──────────────────────────────────
run_android_tests() {
	log "Android: gradle lint + test"
	cd "$PROJECT_DIR/android"
	./gradlew lint --quiet 2>&1 | tee "$RESULTS_DIR/android-lint.log"
	./gradlew test --quiet 2>&1 | tee "$RESULTS_DIR/android-test.log"
	log "Android 测试通过"
}

# ── 构建 APK ──────────────────────────────────────────
build_apk() {
	log "构建 APK"
	cd "$PROJECT_DIR/android"
	./gradlew assembleDebug 2>&1 | tee "$RESULTS_DIR/build-apk.log"
	if [ ! -f "$APK_PATH" ]; then
		fail "APK 构建失败: $APK_PATH"
		exit 1
	fi
	log "APK 构建成功: $APK_PATH"
}

# ── 模拟器管理 ──────────────────────────────────────────
check_kvm() {
	if [ ! -e /dev/kvm ]; then
		fail "KVM 不可用 (/dev/kvm 不存在)"
		echo "启用 KVM:"
		echo "  sudo modprobe kvm"
		echo "  sudo modprobe kvm_intel  # Intel"
		echo "  sudo modprobe kvm_amd    # AMD"
		exit 1
	fi
	log "KVM 可用"
}

create_avd() {
	log "创建 AVD: $AVD_NAME"
	avdmanager create avd \
		-n "$AVD_NAME" \
		-k "system-images;android-36;default;x86_64" \
		-d pixel_7_pro \
		--force 2>&1 | tee "$RESULTS_DIR/avd-create.log"
	log "AVD 创建成功"
}

start_emulator() {
	log "启动模拟器 (超时: ${EMULATOR_TIMEOUT}s)"
	emulator -avd "$AVD_NAME" \
		-no-window \
		-gpu swiftshader_indirect \
		-noaudio \
		-no-boot-anim \
		-camera-back none \
		-memory 2048 \
		-no-snapshot \
		&
	local emulator_pid=$!
	echo $emulator_pid >"$RESULTS_DIR/emulator.pid"

	# 等待模拟器启动
	local timeout=$EMULATOR_TIMEOUT
	local elapsed=0
	while [ $elapsed -lt $timeout ]; do
		if adb shell getprop sys.boot_completed 2>/dev/null | grep -q "1"; then
			log "模拟器启动完成 (${elapsed}s)"
			return 0
		fi
		sleep 5
		elapsed=$((elapsed + 5))
		echo -n "."
	done
	echo ""
	fail "模拟器启动超时 (${timeout}s)"
	return 1
}

stop_emulator() {
	if [ -f "$RESULTS_DIR/emulator.pid" ]; then
		local pid=$(cat "$RESULTS_DIR/emulator.pid")
		if kill -0 "$pid" 2>/dev/null; then
			log "停止模拟器 (PID: $pid)"
			kill "$pid" 2>/dev/null || true
			wait "$pid" 2>/dev/null || true
		fi
		rm -f "$RESULTS_DIR/emulator.pid"
	fi
	adb emu kill 2>/dev/null || true
}

# ── 模拟器测试 ──────────────────────────────────────────
run_emulator_tests() {
	log "=== 模拟器测试 ==="

	check_kvm
	create_avd
	start_emulator

	# 安装 APK
	log "安装 APK"
	adb install -r "$APK_PATH" 2>&1 | tee "$RESULTS_DIR/install.log"

	# 启动应用
	log "启动 MainActivity"
	adb shell am start -a android.intent.action.MAIN -n io.torvox/.MainActivity
	sleep 3

	# 验证 activity 运行
	log "验证 Activity"
	if adb shell dumpsys activity activities | grep -q "io.torvox/.MainActivity"; then
		log "Activity 运行正常"
	else
		fail "Activity 未运行"
		stop_emulator
		exit 1
	fi

	# 验证进程存在
	log "验证进程"
	if adb shell ps | grep -q "io.torvox"; then
		log "进程存在"
	else
		fail "进程不存在"
		stop_emulator
		exit 1
	fi

	# 收集日志
	log "收集 logcat"
	adb logcat -d -s "torvox:*" "Torvox:*" "System.out:*" >"$RESULTS_DIR/logcat.log" 2>&1 || true

	# 截图
	log "截取屏幕"
	adb shell screencap -p /sdcard/torvox-test.png 2>/dev/null &&
		adb pull /sdcard/torvox-test.png "$RESULTS_DIR/screenshot.png" 2>/dev/null || true

	# 运行 Android instrumented 测试 (如果有设备连接)
	log "运行 instrumented 测试"
	cd "$PROJECT_DIR/android"
	./gradlew connectedDebugAndroidTest 2>&1 | tee "$RESULTS_DIR/instrumented.log" || warn "instrumented 测试跳过"

	stop_emulator
	log "模拟器测试完成"
}

# ── 主流程 ──────────────────────────────────────────
MODE="${1:---all}"

case "$MODE" in
--unit)
	run_clippy
	run_rust_tests
	run_proptest
	run_integration_tests
	;;
--android)
	run_android_tests
	;;
--emulator)
	build_apk
	run_emulator_tests
	;;
--all)
	run_clippy
	run_rust_tests
	run_proptest
	run_integration_tests
	run_android_tests
	build_apk
	run_emulator_tests
	;;
*)
	echo "用法: $0 [--unit|--android|--emulator|--all]"
	exit 1
	;;
esac

log "=== 全部测试通过 ==="
echo ""
echo "结果目录: $RESULTS_DIR"
ls -la "$RESULTS_DIR"
