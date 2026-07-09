#!/usr/bin/env python3
"""
Comprehensive Selection Verification — All assertions must PASS

Usage: python3 verify_selection.py
Exit code: 0 = ALL PASS, 1 = FAILURE

Tests:
1. Screenshots are valid 1080x2400 images
2. Word selection handle detected at 64x63px with accent color
3. URL selection handle detected at different position
4. Paste toolbar detected near bottom of screen
5. After-paste content changes visible
"""
from PIL import Image
import sys, os

SCREENSHOTS = os.path.join(os.path.dirname(__file__), "docs", "screenshots")

files = {
    "baseline": "selection-01-baseline.png",
    "word": "selection-02-word-selection.png",
    "url": "selection-03-url-selection.png",
    "paste": "selection-04-paste-button.png",
    "after": "selection-05-after-paste.png",
}

def get_blobs(before, after, th=40, ms=8):
    w, h = min(before.width, after.width), min(before.height, after.height)
    ch = [[False] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            bp = before.getpixel((x, y))[:3]
            ap = after.getpixel((x, y))[:3]
            if sum(abs(bp[i] - ap[i]) for i in range(3)) > th:
                ch[y][x] = True
    vis = [[False] * w for _ in range(h)]
    blobs = []
    for y in range(h):
        for x in range(w):
            if not ch[y][x] or vis[y][x]:
                continue
            x1 = x2 = x
            y1 = y2 = y
            s = [(x, y)]
            vis[y][x] = True
            while s:
                cx, cy = s.pop()
                x1 = min(x1, cx)
                x2 = max(x2, cx)
                y1 = min(y1, cy)
                y2 = max(y2, cy)
                for dx in (-1, 0, 1):
                    for dy in (-1, 0, 1):
                        nx, ny = cx + dx, cy + dy
                        if (
                            0 <= nx < w
                            and 0 <= ny < h
                            and ch[ny][nx]
                            and not vis[ny][nx]
                        ):
                            vis[ny][nx] = True
                            s.append((nx, ny))
            bw, bh = x2 - x1 + 1, y2 - y1 + 1
            if bw >= ms and bh >= ms:
                blobs.append((x1, y1, x2, y2, bw, bh))
    return blobs


def main():
    errors = []
    passed = 0
    failed = 0

    def check(condition, message):
        nonlocal passed, failed
        if condition:
            passed += 1
            print(f"  OK {message}")
        else:
            failed += 1
            errors.append(message)
            print(f"  FAIL {message}")

    print("=" * 60)
    print("COMPREHENSIVE SELECTION VERIFICATION")
    print("=" * 60)

    imgs = {}
    for name, fname in files.items():
        path = os.path.join(SCREENSHOTS, fname)
        if not os.path.exists(path):
            print(f"  FAIL {name}: {fname} not found")
            failed += 1
            continue
        imgs[name] = Image.open(path)
        check(imgs[name].size == (1080, 2400), f"{name}: 1080x2400")

    if "baseline" not in imgs:
        print("FATAL: no baseline image")
        return 1

    base = imgs["baseline"]

    # Word selection
    print("\n--- Word Selection ---")
    if "word" in imgs:
        blobs = get_blobs(base, imgs["word"], th=20)
        handles = [b for b in blobs if 55 <= b[4] <= 70 and 55 <= b[5] <= 70]
        check(len(handles) >= 1, f"Handle detected ({len(handles)})")
        if handles:
            h = handles[0]
            check(60 <= h[4] <= 68, f"Handle width={h[4]} (60-68)")
            check(60 <= h[5] <= 68, f"Handle height={h[5]} (60-68)")
            avg_r, avg_g, avg_b, n = 0, 0, 0, 0
            for y in range(h[1], h[3] + 1):
                for x in range(h[0], h[2] + 1):
                    d = sum(
                        abs(base.getpixel((x, y))[i] - imgs["word"].getpixel((x, y))[i])
                        for i in range(3)
                    )
                    if d > 100:
                        wp = imgs["word"].getpixel((x, y))[:3]
                        avg_r += wp[0]
                        avg_g += wp[1]
                        avg_b += wp[2]
                        n += 1
            if n > 0:
                avg_r /= n
                avg_g /= n
                avg_b /= n
                check(180 <= avg_r <= 220, f"R channel={avg_r:.0f} (180-220)")
                check(130 <= avg_g <= 170, f"G channel={avg_g:.0f} (130-170)")
                check(220 <= avg_b <= 255, f"B channel={avg_b:.0f} (220-255)")
                print(f"         Accent color: R={avg_r:.0f} G={avg_g:.0f} B={avg_b:.0f}")

    # URL selection
    print("\n--- URL Selection ---")
    if "url" in imgs:
        blobs = get_blobs(base, imgs["url"], th=20)
        handles = [b for b in blobs if 55 <= b[4] <= 70 and 55 <= b[5] <= 70]
        check(len(handles) >= 1, f"Handle detected ({len(handles)})")
        if handles:
            h = handles[0]
            check(60 <= h[4] <= 68, f"Handle width={h[4]} (60-68)")
            check(60 <= h[5] <= 68, f"Handle height={h[5]} (60-68)")
        # Verify different position from word handle
        if "word" in imgs:
            wb = get_blobs(base, imgs["word"], th=20)
            wh = [b for b in wb if 55 <= b[4] <= 70 and 55 <= b[5] <= 70]
            if wh and handles:
                check(
                    abs(wh[0][1] - handles[0][1]) > 50,
                    f"Different Y: dy={abs(wh[0][1] - handles[0][1])}",
                )

    # Paste menu
    print("\n--- Paste Menu ---")
    if "paste" in imgs:
        blobs = get_blobs(base, imgs["paste"], th=20, ms=20)
        toolbars = [
            b for b in blobs if b[4] > 200 or b[5] > 60
        ]
        check(len(toolbars) >= 1, f"Large toolbar blob ({len(toolbars)})")
        toolbar_bottom = [
            b for b in blobs if b[1] > 1400 and (b[4] > 200 or b[5] > 50)
        ]
        check(
            len(toolbar_bottom) >= 1,
            f"Toolbar near bottom ({len(toolbar_bottom)})",
        )

    # After paste
    print("\n--- After Paste ---")
    if "after" in imgs:
        blobs = get_blobs(base, imgs["after"], th=20)
        check(len(blobs) > 0, f"Changes detected ({len(blobs)} blobs)")

    # Summary
    print(f"\n{'=' * 60}")
    print(f"RESULTS: {passed} passed, {failed} failed ({passed + failed} total)")
    for e in errors:
        print(f"  FAIL {e}")
    print(f"EXIT: {'ALL PASSED' if failed == 0 else 'FAILURES DETECTED'}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
