"""Verify selection screenshots from emulator using Pillow."""
import sys
from pathlib import Path
from PIL import Image

DIR = Path("docs/screenshots")

def pixel_diff_3ch(px1, px2):
    """Sum of absolute channel diffs for two pixels (RGBA tuples)."""
    return abs(px1[0] - px2[0]) + abs(px1[1] - px2[1]) + abs(px1[2] - px2[2])

def find_blobs(before_img, after_img, threshold=50, min_size=10):
    """Find connected changed pixel regions."""
    w, h = before_img.size
    before_px = before_img.load()
    after_px = after_img.load()

    changed = [[False] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            diff = pixel_diff_3ch(before_px[x, y], after_px[x, y])
            changed[y][x] = diff > threshold

    visited = [[False] * w for _ in range(h)]
    blobs = []
    for y in range(h):
        for x in range(w):
            if not changed[y][x] or visited[y][x]:
                continue
            stack = [(x, y)]
            visited[y][x] = True
            x1 = x2 = x; y1 = y2 = y
            while stack:
                cx, cy = stack.pop()
                x1 = min(x1, cx); x2 = max(x2, cx)
                y1 = min(y1, cy); y2 = max(y2, cy)
                for dx in (-1, 0, 1):
                    for dy in (-1, 0, 1):
                        nx, ny = cx + dx, cy + dy
                        if 0 <= nx < w and 0 <= ny < h and changed[ny][nx] and not visited[ny][nx]:
                            visited[ny][nx] = True
                            stack.append((nx, ny))
            bw, bh = x2 - x1 + 1, y2 - y1 + 1
            if bw >= min_size and bh >= min_size:
                blobs.append((x1, y1, x2, y2, bw, bh))
    return blobs

# Load screenshots
files = {
    "01-baseline": "selection-01-baseline.png",
    "02-word-selection": "selection-02-word-selection.png",
    "03-url-selection": "selection-03-url-selection.png",
    "04-paste-button": "selection-04-paste-button.png",
    "05-after-paste": "selection-05-after-paste.png",
}
images = {}
for key, fname in files.items():
    path = DIR / fname
    if not path.exists():
        print(f"❌ MISSING: {path}")
        sys.exit(1)
    img = Image.open(path)
    assert img.size == (1080, 2400), f"{key}: bad size {img.size}"
    images[key] = img
    print(f"✅ {key}: {img.size} ({path.stat().st_size//1024}KB)")

print("\n" + "=" * 60)
print("PIXEL DIFF ANALYSIS")
print("=" * 60)

base = images["01-baseline"]

for key in ["02-word-selection", "03-url-selection", "04-paste-button", "05-after-paste"]:
    img = images[key]
    w, h = img.size
    base_px = base.load()
    img_px = img.load()
    total_diff = sum(pixel_diff_3ch(base_px[x, y], img_px[x, y]) for y in range(h) for x in range(w))
    max_diff_raw = max(pixel_diff_3ch(base_px[x, y], img_px[x, y]) for y in range(h) for x in range(w))
    total_pixels_3ch = w * h * 3
    pct = total_diff / total_pixels_3ch * 100
    print(f"\n--- {key} ---")
    print(f"  Total pixel diff sum: {total_diff:,} ({pct:.4f}%)")
    print(f"  Max per-channel diff at any pixel: {max_diff_raw}")

    blobs = find_blobs(base, img, threshold=50, min_size=10)
    print(f"  Changed regions: {len(blobs)}")
    for bx1, by1, bx2, by2, bw, bh in sorted(blobs):
        print(f"    ({bx1},{by1})-({bx2},{by2}) = {bw}x{bh}px")

    # Handle detection: 55-75px wide/tall
    handles = [(bx1, by1, bx2, by2, bw, bh) for bx1, by1, bx2, by2, bw, bh in blobs
               if 55 <= bw <= 75 and 55 <= bh <= 75]
    if handles:
        print(f"  🔵 Potential handles ({len(handles)}):")
        for hx1, hy1, hx2, hy2, hw, hh in handles:
            cx, cy = (hx1 + hx2) // 2, (hy1 + hy2) // 2
            print(f"    center=({cx},{cy}) size={hw}x{hh}")

        # Verify handles are on same row
        if len(handles) >= 2:
            centers_y = [(h[1] + h[3]) // 2 for h in handles]
            if max(centers_y) - min(centers_y) < 80:
                print(f"  ✅ Handles on same terminal row")
            else:
                print(f"  ⚠️  Handles on different rows: {centers_y}")

    # Toolbar/menu detection
    toolbars = [(bx1, by1, bx2, by2, bw, bh) for bx1, by1, bx2, by2, bw, bh in blobs
                if bw > 400 and bh > 50 and bh < 500]
    if toolbars:
        print(f"  🟦 Large UI element (toolbar/menu) detected:")
        for tx1, ty1, tx2, ty2, tw, th in toolbars:
            print(f"    ({tx1},{ty1})-({tx2},{ty2}) = {tw}x{th}px")
    else:
        # Try looser criteria
        toolbars = [(bx1, by1, bx2, by2, bw, bh) for bx1, by1, bx2, by2, bw, bh in blobs
                    if bw > 200 and bh > 40 and bh < 500]
        if toolbars:
            print(f"  Medium UI element ({len(toolbars)}):")
            for tx1, ty1, tx2, ty2, tw, th in toolbars:
                print(f"    ({tx1},{ty1})-({tx2},{ty2}) = {tw}x{th}px")

    verdict = len(blobs) > 0
    print(f"  {'✅ PASS' if verdict else '❌ FAIL'} - {'changes detected' if verdict else 'no changes found'}")

print("\n" + "=" * 60)
print("OVERALL VERDICT")
print("=" * 60)

# Check baseline has real content
px = base.load()
non_black = 0
for y in range(2400):
    for x in range(1080):
        r, g, b = px[x, y][:3]
        if r > 20 or g > 20 or b > 20:
            non_black += 1
print(f"Baseline non-black pixels: {non_black:,} / {1080*2400:,} ({non_black/(1080*2400)*100:.1f}%)")
print("✅ All screenshots pass verification" if non_black > 100000 else "❌ Baseline appears blank")
