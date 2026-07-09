"""Verification of 5-stage emulator test screenshots"""
import subprocess, sys, os, json
from PIL import Image

errors = []
info = []

def check(cond, msg):
    if cond:
        info.append(f"  OK: {msg}")
    else:
        errors.append(f"  FAIL: {msg}")

def img_diff(a, b):
    """Return pixel diff between two images"""
    w, h = min(a.size[0], b.size[0]), min(a.size[1], b.size[1])
    ap, bp = a.load(), b.load()
    changed = set()
    for y in range(h):
        for x in range(w):
            pa = ap[x, y][:3] if len(ap[x, y]) >= 3 else ap[x, y]
            pb = bp[x, y][:3] if len(bp[x, y]) >= 3 else bp[x, y]
            if any(abs(pa[c] - pb[c]) > 40 for c in range(3)):
                changed.add((x, y))
    return changed

def find_blobs(pixels, min_w=8, min_h=8):
    """Cluster connected changed pixels into blobs"""
    w, h = 1080, 2400
    visited = set()
    blobs = []
    for px, py in pixels:
        if (px, py) in visited: continue
        stack = [(px, py)]
        visited.add((px, py))
        x1 = x2 = px; y1 = y2 = py
        while stack:
            cx, cy = stack.pop()
            x1 = min(x1, cx); x2 = max(x2, cx)
            y1 = min(y1, cy); y2 = max(y2, cy)
            for dx in (-1,0,1):
                for dy in (-1,0,1):
                    nx, ny = cx+dx, cy+dy
                    if (nx, ny) in pixels and (nx, ny) not in visited:
                        visited.add((nx, ny))
                        stack.append((nx, ny))
        bw, bh = x2-x1+1, y2-y1+1
        if bw >= min_w and bh >= min_h:
            blobs.append((x1, y1, x2, y2, bw, bh))
    return blobs

base = "docs/screenshots"

print("=" * 60)
print("TORVOX PIPELINE VERIFICATION")
print("=" * 60)

# 1. Check files exist and have reasonable sizes
files = {
    "em_01_baseline.png": ("Baseline", 40),
    "em_02_typed.png": ("Typed", 40),
    "em_03_word.png": ("Word selection", 70),
    "em_04_url.png": ("URL selection", 70),
    "em_05_paste.png": ("Paste", 70),
}

images = {}
for fn, (label, min_kb) in files.items():
    path = os.path.join(base, fn)
    if not os.path.exists(path):
        check(False, f"{label}: file not found")
        continue
    size_kb = os.path.getsize(path) / 1024
    check(size_kb >= min_kb, f"{label}: {size_kb:.0f}KB >= {min_kb}KB")
    im = Image.open(path).convert("RGB")
    check(im.size == (1080, 2400), f"{label}: resolution {im.size[0]}x{im.size[1]}")
    images[fn] = im

if not images:
    print("No images to analyze. Aborting.")
    sys.exit(1)

# 2. Word vs baseline: look for handle blobs
print("\n--- Word selection vs baseline ---")
if "em_01_baseline.png" in images and "em_03_word.png" in images:
    changed = img_diff(images["em_01_baseline.png"], images["em_03_word.png"])
    blobs = find_blobs(changed, 50, 50)
    check(len(blobs) >= 1, f"Found {len(blobs)} handle-sized blobs")
    for i, b in enumerate(blobs):
        info.append(f"  blob[{i}]: ({b[0]},{b[1]})-({b[2]},{b[3]}) = {b[4]}x{b[5]}")
    # Check for accent-colored pixels in changed region
    accent = 0
    for x, y in changed:
        r, g, b = images["em_03_word.png"].getpixel((x, y))
        if r > 180 and g < 190 and b > 210:
            accent += 1
    check(accent > 10, f"Found {accent} accent-colored pixels in changed region")

# 3. URL vs baseline
print("\n--- URL selection vs baseline ---")
if "em_01_baseline.png" in images and "em_04_url.png" in images:
    changed = img_diff(images["em_01_baseline.png"], images["em_04_url.png"])
    blobs = find_blobs(changed, 50, 50)
    check(len(blobs) >= 1, f"Found {len(blobs)} handle-sized blobs")
    for i, b in enumerate(blobs):
        info.append(f"  blob[{i}]: ({b[0]},{b[1]})-({b[2]},{b[3]}) = {b[4]}x{b[5]}")

# 4. Paste button (vs baseline)
print("\n--- Paste vs baseline ---")
if "em_01_baseline.png" in images and "em_05_paste.png" in images:
    changed = img_diff(images["em_01_baseline.png"], images["em_05_paste.png"])
    blobs = find_blobs(changed, 50, 50)
    check(len(blobs) >= 2, f"Found {len(blobs)} blobs (expected handles + toolbar)")
    # Check for large horizontal toolbar
    toolbars = [b for b in blobs if b[4] > 200 and b[5] < 100]
    check(len(toolbars) >= 1, f"Found {len(toolbars)} toolbar-sized blobs")

# 5. Word vs typed (isolates selection changes from typing)
print("\n--- Word selection vs typed ---")
if "em_02_typed.png" in images and "em_03_word.png" in images:
    changed = img_diff(images["em_02_typed.png"], images["em_03_word.png"])
    check(len(changed) > 100, f"Pixel changes: {len(changed)} (selection overlay + handles)")

# 6. Verify baseline has content
print("\n--- Baseline content check ---")
if "em_01_baseline.png" in images:
    im = images["em_01_baseline.png"]
    bright = sum(1 for y in range(0, 2400, 20) for x in range(1050, 1080)
                 if max(im.getpixel((x, y))) > 50)
    check(bright > 1000, f"Right strip brightness: {bright} (expect >1000 for scrollbar)")

# 7. Check the test completed via logcat
print("\n--- Test completion ---")
result = subprocess.run(
    ["adb", "exec-out", "logcat", "-d"],
    capture_output=True, text=True, timeout=5
)
for line in result.stdout.split("\n"):
    if "STAGE" in line or "ALL_STAGES" in line:
        print(f"  Log: {line.strip()}")

print(f"\n{'=' * 60}")
print(f"RESULTS: {len(info)} passed, {len(errors)} failed")
if errors:
    print("ERRORS:")
    for e in errors:
        print(f"  {e}")
else:
    print("ALL CHECKS PASSED")
print(f"{'=' * 60}")

# Export results for commit
results = {"passed": len(info), "failed": len(errors), "details": info + errors}
with open(os.path.join(base, "pipeline_results.json"), "w") as f:
    json.dump(results, f)

sys.exit(0 if not errors else 1)
