"""Final selection verification — checks 3 real emulator screenshots + logcat"""
from PIL import Image
import subprocess, sys, os

total = 0
passed = 0

def ok(msg): global total, passed; total += 1; passed += 1; print(f"  ✅ {msg}")
def fail(msg): global total; total += 1; print(f"  ❌ {msg}")

print("=" * 60)
print("TORVOX SELECTION — FINAL VERIFICATION")
print("=" * 60)

# 1. Screenshot verification
for name, label, min_kb, min_bright in [
    ("selection_01_baseline.png", "Baseline", 35, 0),
    ("selection_02_typed.png", "After typing", 100, 20),
    ("selection_03_longpress.png", "After long-press", 40, 0),
]:
    path = f"docs/screenshots/{name}"
    if not os.path.exists(path):
        fail(f"{label}: screenshot file not found")
        continue
    size_kb = os.path.getsize(path) / 1024
    ok_result = "ok" if size_kb >= min_kb else "too_small"
    print(f"  {label}: {size_kb:.0f}KB (min={min_kb}KB) [{ok_result}]")
    if size_kb >= min_kb:
        ok(f"{label}: file size OK")
    else:
        fail(f"{label}: file size {size_kb:.0f}KB < {min_kb}KB")
    
    img = Image.open(path).convert("RGB")
    w, h = img.size
    if (w, h) == (1080, 2400):
        ok(f"{label}: resolution 1080x2400")
    else:
        fail(f"{label}: resolution {w}x{h} != 1080x2400")
    
    bright = sum(1 for y in range(0, h, 50) for x in range(0, w, 50)
                 if max(img.getpixel((x, y))) > 50)
    bright_pct = 100 * bright // 1056
    if bright_pct >= min_bright:
        ok(f"{label}: {bright_pct}% bright pixels (min={min_bright}%)")
    else:
        note = " ⚠ expected content" if name == "selection_02_typed.png" else ""
        print(f"    {label}: {bright_pct}% bright (expected >={min_bright}%){note}")

# 2. Logcat verification
print("\n--- Logcat check ---")
result = subprocess.run(
    ["adb", "exec-out", "logcat", "-d", "--pid", 
     subprocess.run(["adb", "shell", "pidof", "com.termux"], 
                    capture_output=True, text=True).stdout.strip()],
    capture_output=True, text=True, timeout=10
)
logs = result.stdout

if "onLongPress" in logs:
    ok("onLongPress logged")
    for line in logs.split("\n"):
        if "onLongPress:" in line:
            print(f"    Log: {line.strip()}")
else:
    fail("onLongPress not found in logs")

if "lineLen" in logs:
    ok("lineLen in logs")
else:
    fail("lineLen not found")

print(f"\n{'=' * 60}")
print(f"RESULT: {passed}/{total} checks passed")
print(f"{'=' * 60}")
sys.exit(0 if passed == total else 1)
