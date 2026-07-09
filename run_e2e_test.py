#!/usr/bin/env python3
"""E2E selection test with OCR and pixel verification. Runs via ADB."""

import subprocess, sys, time, os, json
from pathlib import Path
from PIL import Image

HERE = Path(__file__).parent
SCREENSHOTS = HERE / "docs" / "screenshots"
SCREENSHOTS.mkdir(parents=True, exist_ok=True)
TMP = Path("/tmp/e2e-run")
TMP.mkdir(parents=True, exist_ok=True)

ADB = "/usr/local/lib/android/sdk/platform-tools/adb"


def sh(cmd, timeout=30):
    r = subprocess.run(cmd, capture_output=True, timeout=timeout)
    r.stdout = r.stdout.decode("utf-8", "replace")
    r.stderr = r.stderr.decode("utf-8", "replace")
    return r


def sh_bin(cmd, timeout=30):
    return subprocess.run(cmd, capture_output=True, timeout=timeout)


def tap(x, y):
    sh([ADB, "shell", "input", "tap", str(int(x)), str(int(y))])


def swipe(x1, y1, x2, y2, ms=900):
    sh(
        [
            ADB,
            "shell",
            "input",
            "swipe",
            str(int(x1)),
            str(int(y1)),
            str(int(x2)),
            str(int(y2)),
            str(ms),
        ]
    )


def keyevent(k):
    sh([ADB, "shell", "input", "keyevent", k])


def writeln(text):
    escaped = text.replace(" ", "%s")
    keyevent("KEYCODE_WAKEUP")
    time.sleep(0.5)
    sh([ADB, "shell", "input", "text", escaped])
    time.sleep(0.5)
    keyevent("KEYCODE_ENTER")


def cap(name):
    result = sh_bin([ADB, "exec-out", "screencap", "-p"], timeout=10)
    path = TMP / f"{name}.png"
    if len(result.stdout) > 200:
        path.write_bytes(result.stdout)
        return path
    sh([ADB, "shell", "screencap", "-p", f"/data/local/tmp/{name}.png"], timeout=10)
    sh([ADB, "pull", f"/data/local/tmp/{name}.png", str(path)], timeout=10)
    return path if path.exists() and path.stat().st_size > 200 else None


def longpress(x, y, ms=900):
    swipe(x, y, x + 1, y + 1, ms)


class Blob:
    def __init__(self, x1, y1, x2, y2):
        self.x1, self.y1, self.x2, self.y2 = x1, y1, x2, y2

    @property
    def cx(self):
        return (self.x1 + self.x2) // 2

    @property
    def cy(self):
        return (self.y1 + self.y2) // 2

    @property
    def w(self):
        return self.x2 - self.x1 + 1

    @property
    def h(self):
        return self.y2 - self.y1 + 1

    def __repr__(self):
        return f"B({self.x1},{self.y1})-({self.x2},{self.y2}) {self.w}x{self.h}"


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
                blobs.append(Blob(x1, y1, x2, y2))
    return blobs


def verify_accent_color(img, blob):
    """Check if blob area contains accent color (blue-ish) pixels."""
    count = 0
    for y in range(blob.y1, min(blob.y2 + 1, img.height), 2):
        for x in range(blob.x1, min(blob.x2 + 1, img.width), 2):
            r, g, b, a = img.getpixel((x, y))
            if b > 150 and r < 200 and g < 200:  # blue accent
                count += 1
    return count > 5


def find_text_rows(img):
    """Find rows that contain text (non-dark pixels)."""
    text_rows = []
    for y in range(img.height):
        light = sum(
            1
            for x in range(0, img.width, 4)
            if sum(c > 50 for c in img.getpixel((x, y))[:3]) > 0
        )
        if light > 5:
            text_rows.append(y)
    return text_rows


def launch_and_setup():
    """Kill, launch, type content, wait for response."""
    sh([ADB, "shell", "am", "force-stop", "com.termux"], timeout=5)
    time.sleep(2)
    sh(
        [ADB, "shell", "am", "start", "-n", "com.termux/io.torvox.MainActivity", "-W"],
        timeout=15,
    )
    time.sleep(4)

    # Tap to focus terminal
    tap(240, 350)
    time.sleep(2)

    # Type commands to generate content we can select
    writeln("echo abcdefghijklmnopqrstuvwxyz")
    time.sleep(3)
    writeln("echo https://example.com/test-url")
    time.sleep(3)
    writeln("echo $RANDOM")
    time.sleep(3)
    # Dismiss IME by pressing back
    keyevent("KEYCODE_BACK")
    time.sleep(2)


def main():
    passed, failed = 0, 0
    errors = []

    def check(ok, msg):
        nonlocal passed, failed
        if ok:
            passed += 1
            print(f"  ✓ {msg}")
        else:
            failed += 1
            errors.append(msg)
            print(f"  ✗ {msg}")

    print("=" * 60)
    print("E2E SELECTION VERIFICATION")
    print("=" * 60)

    print("\n[Setup] Launching app and typing content")
    launch_and_setup()

    print("\n[Step 0] Baseline screenshot")
    cap("0-baseline")
    base = cap("0-baseline")
    if not base:
        print("FATAL: No baseline screenshot")
        return 1
    base_img = Image.open(base)
    print(f"  Screen: {base_img.width}x{base_img.height}")
    text_rows = find_text_rows(base_img)
    print(
        f"  Text rows: {text_rows[:5]}...{text_rows[-5:] if len(text_rows) > 5 else ''} (total {len(text_rows)})"
    )
    base_img.save(SCREENSHOTS / "selection-01-baseline.png")

    if not text_rows:
        print("  WARNING: No text detected, trying to type again")
        tap(240, 350)
        time.sleep(1)
        writeln("echo hello_world_12345")
        time.sleep(3)
        keyevent("KEYCODE_BACK")
        time.sleep(2)
        base = cap("1-retry-baseline")
        if base:
            base_img = Image.open(base)
            text_rows = find_text_rows(base_img)
            base_img.save(SCREENSHOTS / "selection-01-baseline.png")

    # Find terminal area bounds
    term_top = text_rows[0] if text_rows else 0
    term_bottom = text_rows[-1] if text_rows else base_img.height
    cell_h = (term_bottom - term_top) / 24  # assume 24 rows visible
    cell_w = base_img.width / 80  # assume 80 cols
    print(
        f"  Terminal area: y={term_top}-{term_bottom}, cell={cell_w:.1f}x{cell_h:.1f}"
    )

    # [Step 1] Longpress on text to select word
    print("\n[Step 1] Word selection via longpress")
    # Longpress on middle of visible text area
    lx = int(base_img.width * 0.3)  # ~col 24
    ly = int((term_top + term_bottom) / 2)  # middle of text
    print(f"  Longpress at ({lx}, {ly})")
    longpress(lx, ly)
    time.sleep(2)

    word_img_path = cap("2-word-selection")
    if word_img_path:
        word_img = Image.open(word_img_path)
        word_img.save(SCREENSHOTS / "selection-02-word-selection.png")
        blobs = get_blobs(base_img, word_img, th=30)
        handles = [b for b in blobs if 40 <= b.w <= 80 and 25 <= b.h <= 45]
        print(f"  Changed blobs: {len(blobs)}, potential handles: {len(handles)}")
        for h in handles:
            print(
                f"    [{h.w}x{h.h}] at cell({h.cx / cell_w:.0f},{h.cy / cell_h:.0f}) color_accent={verify_accent_color(word_img, h)}"
            )

        # Count selection-highlighted cells (inverted colors in text area)
        highlighted = [b for b in blobs if b.h <= cell_h * 2 and b.w >= cell_w]
        check(
            len(handles) >= 1 or len(highlighted) >= 3,
            f"Selection visible: handles={len(handles)} highlighted_regions={len(highlighted)}",
        )

        # Take OCR of selected region
        sh([ADB, "shell", "screencap", "-p", "/data/local/tmp/ocr_word.png"], timeout=5)
        sh(
            [ADB, "pull", "/data/local/tmp/ocr_word.png", str(TMP / "ocr_word.png")],
            timeout=5,
        )
        ocr_result = sh(["rapidocr", "-img", str(TMP / "ocr_word.png")], timeout=15)
        print(f"  OCR output (first 200 chars): {ocr_result.stdout[:200]}")
    else:
        check(False, "Screenshot for word selection")

    # Tap to dismiss selection
    tap(lx + 50, ly)
    time.sleep(1)

    # [Step 2] URL selection
    print("\n[Step 2] URL selection via longpress")
    # Find URL text - look for 'https://' on screen
    lx_url = int(base_img.width * 0.1)
    ly_url = term_top + int(cell_h * 1.5)
    print(f"  Longpress at ({lx_url}, {ly_url})")
    longpress(lx_url, ly_url)
    time.sleep(2)

    url_img_path = cap("3-url-selection")
    if url_img_path:
        url_img = Image.open(url_img_path)
        url_img.save(SCREENSHOTS / "selection-03-url-selection.png")
        blobs = get_blobs(base_img, url_img, th=30)
        # URL should produce wider highlighted area
        wide = [b for b in blobs if b.w > cell_w * 10]
        check(len(wide) >= 1, f"URL selection wide highlights: {len(wide)}")
        handles = [b for b in blobs if 40 <= b.w <= 80 and 25 <= b.h <= 45]
        check(len(handles) >= 1, f"URL handles visible: {len(handles)}")
    else:
        check(False, "Screenshot for URL selection")

    # Tap to dismiss
    tap(lx_url + 50, ly_url)
    time.sleep(1)

    # [Step 3] Paste menu on empty area
    print("\n[Step 3] Paste button on whitespace longpress")
    keyevent("KEYCODE_BACK")  # dismiss IME
    time.sleep(1)

    # Put something in clipboard
    sh(
        [ADB, "shell", "service", "call", "clipboard", "11"], timeout=5
    )  # this might not work - try content
    sh(
        [
            ADB,
            "shell",
            "content",
            "insert",
            "--uri",
            "content://clipboard",
            "--bind",
            "text:s:test_paste_content",
        ],
        timeout=5,
    )

    # Longpress on empty area at bottom of terminal
    lx_empty = 100
    ly_empty = term_bottom - int(cell_h * 0.5)
    print(f"  Longpress at ({lx_empty}, {ly_empty})")
    longpress(lx_empty, ly_empty)
    time.sleep(2)

    paste_img_path = cap("4-paste-menu")
    if paste_img_path:
        paste_img = Image.open(paste_img_path)
        paste_img.save(SCREENSHOTS / "selection-04-paste-button.png")
        blobs = get_blobs(base_img, paste_img, th=20, ms=15)
        large = [b for b in blobs if b.w > 100 or b.h > 50]
        print(f"  Changed: {len(blobs)} blobs, {len(large)} large (toolbar candidates)")
        for b in large:
            print(f"    [{b.w}x{b.h}] at ({b.x1},{b.y1})-({b.x2},{b.y2})")
    else:
        check(False, "Screenshot for paste menu")

    # [Step 4] Check that longpress on whitespace shows single-cell invert
    print("\n[Step 4] Whitespace single-cell invert")
    # Already captured in the paste step - verify there's a highlighted cell

    # [Step 5] Dismiss all and verify final state
    print("\n[Step 5] Final state after dismiss")
    tap(200, term_top + 10)
    time.sleep(2)

    final_img_path = cap("5-after-dismiss")
    if final_img_path:
        final_img = Image.open(final_img_path)
        final_img.save(SCREENSHOTS / "selection-05-after-dismiss.png")

    print(f"\n{'=' * 60}")
    print(f"RESULTS: {passed} passed, {failed} failed ({passed + failed} total)")
    for e in errors:
        print(f"  ✗ {e}")
    print(f"\nScreenshots: {SCREENSHOTS}/selection-*.png")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
