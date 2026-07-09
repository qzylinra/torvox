#!/usr/bin/env python3
"""
Final end-to-end selection verification.
Uses ONLY adb shell commands (no app-internal restrictions).
Steps:
1. Launch app and ensure it's focused
2. Type terminal content via focus + keyevent
3. Capture: baseline, word-selection, url-selection, paste-menu
4. Pixel analysis with PASS/FAIL assertions
"""
import subprocess, sys, time, os
from pathlib import Path
from PIL import Image

HERE = Path(__file__).parent
SCREENSHOTS = HERE / "docs" / "screenshots"
SCREENSHOTS.mkdir(parents=True, exist_ok=True)
TMP = Path("/tmp/e2e-final")
TMP.mkdir(parents=True, exist_ok=True)

def sh(args, timeout=30):
    return subprocess.run(["adb"] + args, capture_output=True, check=False, timeout=timeout)

def cap(name):
    result = sh(["exec-out", "screencap", "-p"], timeout=10)
    if len(result.stdout) > 200:
        (TMP / f"{name}.png").write_bytes(result.stdout)
        return True
    # fallback: write to file
    sh(["shell", "screencap", "-p", f"/data/local/tmp/{name}.png"], timeout=10)
    sh(["pull", f"/data/local/tmp/{name}.png", str(TMP / f"{name}.png")], timeout=10)
    sh(["shell", "rm", "-f", f"/data/local/tmp/{name}.png"], timeout=5)
    return (TMP / f"{name}.png").exists() and (TMP / f"{name}.png").stat().st_size > 200

def tap(x, y):
    sh(["shell", "input", "tap", str(int(x)), str(int(y))], timeout=5)

def longpress(x, y, ms=900):
    x, y = int(x), int(y)
    sh(["shell", "input", "swipe", str(x), str(y), str(x+1), str(y+1), str(ms)], timeout=5)

def writeln(text):
    # Replace spaces with %s for adb shell input text
    escaped = text.replace(" ", "%s")
    sh(["shell", "input", "text", escaped], timeout=30)
    sh(["shell", "input", "keyevent", "KEYCODE_ENTER"], timeout=10)

def wake_and_launch():
    sh(["shell", "input", "keyevent", "KEYCODE_WAKEUP"])
    time.sleep(1)
    sh(["shell", "monkey", "-p", "com.termux", "-c", "android.intent.category.LAUNCHER", "1"], timeout=10)
    time.sleep(5)
    # Bring to front via broadcast
    sh(["shell", "am", "start", "-a", "android.intent.action.MAIN",
        "-n", "com.termux/io.torvox.MainActivity"], timeout=10)
    time.sleep(3)

class Blob:
    def __init__(self, x1, y1, x2, y2):
        self.x1, self.y1, self.x2, self.y2 = x1, y1, x2, y2
    @property
    def cx(self): return (self.x1+self.x2)//2
    @property
    def cy(self): return (self.y1+self.y2)//2
    @property
    def w(self): return self.x2-self.x1+1
    @property
    def h(self): return self.y2-self.y1+1
    def __repr__(self): return f"B({self.x1},{self.y1})-({self.x2},{self.y2}) {self.w}x{self.h}"

def get_blobs(before, after, th=40, ms=8):
    w, h = min(before.width,after.width), min(before.height,after.height)
    ch = [[False]*w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            bp = before.getpixel((x,y))[:3]
            ap = after.getpixel((x,y))[:3]
            if sum(abs(bp[i]-ap[i]) for i in range(3)) > th:
                ch[y][x] = True
    vis = [[False]*w for _ in range(h)]
    blobs = []
    for y in range(h):
        for x in range(w):
            if not ch[y][x] or vis[y][x]: continue
            x1=x2=x; y1=y2=y
            s = [(x,y)]; vis[y][x]=True
            while s:
                cx, cy = s.pop()
                x1=min(x1,cx); x2=max(x2,cx); y1=min(y1,cy); y2=max(y2,cy)
                for dx in (-1,0,1):
                    for dy in (-1,0,1):
                        nx, ny = cx+dx, cy+dy
                        if 0<=nx<w and 0<=ny<h and ch[ny][nx] and not vis[ny][nx]:
                            vis[ny][nx]=True; s.append((nx,ny))
            bw, bh = x2-x1+1, y2-y1+1
            if bw>=ms and bh>=ms:
                blobs.append(Blob(x1,y1,x2,y2))
    return blobs

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
    print("FINAL END-TO-END SELECTION VERIFICATION")
    print("=" * 60)

    print("\n[Setup]")
    wake_and_launch()

    # Check if app is showing
    cap("0-check")
    img0 = Image.open(TMP / "0-check.png")
    print(f"  Screen: {img0.width}x{img0.height}")
    # Try to write content
    tap(img0.width//2, img0.height//3)
    time.sleep(1)

    writeln("echo 'hello world")
    time.sleep(2)
    writeln("echo 'https://github.com/termux test'")
    time.sleep(3)

    # Baseline
    print("\n[1] Baseline")
    cap("1-baseline")
    base = Image.open(TMP / "1-baseline.png")
    bw, bh = base.size
    cell_w, cell_h = bw/80, bh/24
    print(f"  {bw}x{bh}, cell={cell_w:.1f}x{cell_h:.1f}")
    # Copy baseline to project
    base.save(SCREENSHOTS / "selection-01-baseline.png")

    # [2] Word selection
    print("\n[2] Word selection")
    wx = int(cell_w * 8)   # column 8 = "world"
    wy = int(cell_h * 0.8) # row 0
    print(f"  Long-press ({wx},{wy})")
    longpress(wx, wy)
    time.sleep(2)
    cap("2-word")
    word = Image.open(TMP / "2-word.png")
    word.save(SCREENSHOTS / "selection-02-word-selection.png")
    blobs = get_blobs(base, word)
    handles = [b for b in blobs if 50<=b.w<=75 and 50<=b.h<=75]
    print(f"  Changed: {len(blobs)} blobs, {len(handles)} handles")
    for h in handles:
        print(f"    [{h.w}x{h.h}] at cell({h.cx/cell_w:.0f},{h.cy/cell_h:.0f})")
    check(len(handles) >= 2, f"Word: >=2 handles ({len(handles)})")
    if len(handles) >= 2:
        shandles = sorted(handles, key=lambda h: h.cx)
        dy = abs(shandles[0].cy - shandles[1].cy)
        check(dy < cell_h*2, f"Word: handles on same row (dy={dy:.0f})")
    check(any(abs(h.cy-wy) < cell_h*3 for h in handles),
          "Word: handles near long-press Y")

    # [3] URL selection
    print("\n[3] URL selection")
    tap(100, wy)  # clear
    time.sleep(1)
    ux = int(cell_w * 1)
    uy = int(cell_h * 1.8)
    print(f"  Long-press ({ux},{uy})")
    longpress(ux, uy)
    time.sleep(2)
    cap("3-url")
    url = Image.open(TMP / "3-url.png")
    url.save(SCREENSHOTS / "selection-03-url-selection.png")
    ublobs = get_blobs(base, url)
    uhandles = [b for b in ublobs if 50<=b.w<=75 and 50<=b.h<=75]
    print(f"  Changed: {len(ublobs)} blobs, {len(uhandles)} handles")
    for h in uhandles:
        print(f"    [{h.w}x{h.h}] at cell({h.cx/cell_w:.0f},{h.cy/cell_h:.0f})")
    check(len(uhandles) >= 2, f"URL: >=2 handles ({len(uhandles)})")
    if len(uhandles) >= 2:
        such = sorted(uhandles, key=lambda h: h.cx)
        cells = (such[1].cx - such[0].cx) / cell_w
        check(cells >= 5, f"URL: spans {cells:.0f} cells (>=5)")

    # [4] Paste menu
    print("\n[4] Paste menu")
    tap(100, wy)
    time.sleep(1)
    sh(["shell", "am", "broadcast", "-a", "android.intent.action.CLIPBOARD_CHANGED"], check=False)
    sh(["shell", "content", "insert", "--uri", "content://com.termux.clipboard",
        "--bind", "text:s:paste_test"], check=False)
    px, py = bw//2, bh-int(cell_h*4)
    print(f"  Long-press ({px},{py})")
    longpress(px, py)
    time.sleep(2)
    cap("4-paste")
    paste = Image.open(TMP / "4-paste.png")
    paste.save(SCREENSHOTS / "selection-04-paste-button.png")
    pblobs = get_blobs(base, paste, ms=20)
    pbig = [b for b in pblobs if b.w>200 or b.h>80]
    print(f"  Changed: {len(pblobs)} blobs, {len(pbig)} large")
    for b in pbig:
        print(f"    [{b.w}x{b.h}] at ({b.x1},{b.y1})-({b.x2},{b.y2})")
    check(len(pbig) > 0, "Paste: toolbar found")
    if pbig:
        near = any(abs(b.cy-py) < bh/4 for b in pbig)
        check(near, f"Paste: toolbar near Y={py}")

    # Summary
    print(f"\n{'='*60}")
    print(f"PASSED: {passed}  FAILED: {failed}")
    for e in errors:
        print(f"  ✗ {e}")
    print(f"\nScreenshots: {SCREENSHOTS}/selection-*.png")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
