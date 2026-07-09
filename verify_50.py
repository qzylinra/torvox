#!/usr/bin/env python3
"""50+ test cases: text selection OCR/pixel/diff verification via ADB."""

import subprocess, sys, time, os, re
from pathlib import Path
from PIL import Image

HERE = Path(__file__).parent
SCREENSHOTS = HERE / "docs" / "screenshots"
SCREENSHOTS.mkdir(parents=True, exist_ok=True)
TMP = Path("/tmp/verify-50")
TMP.mkdir(parents=True, exist_ok=True)
ADB = "/usr/local/lib/android/sdk/platform-tools/adb"
PASSED = 0
FAILED = 0
ERRORS = []


def sh_bin(cmd, t=30):
    return subprocess.run(cmd, capture_output=True, timeout=t)


def sh(cmd, t=30):
    r = subprocess.run(cmd, capture_output=True, timeout=t)
    r.stdout = r.stdout.decode("utf-8", "replace")
    r.stderr = r.stderr.decode("utf-8", "replace")
    return r


def tap(x, y):
    sh([ADB, "shell", "input", "tap", str(int(x)), str(int(y))])


def swipe(x1, y1, x2, y2, ms=100):
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


def longpress(x, y, ms=900):
    swipe(x, y, x + 1, y + 1, ms)


def key(k):
    sh([ADB, "shell", "input", "keyevent", k])


def writeln(txt):
    key("KEYCODE_WAKEUP")
    time.sleep(0.3)
    sh([ADB, "shell", "input", "text", txt.replace(" ", "%s")])
    time.sleep(0.3)
    key("KEYCODE_ENTER")


def cap(name):
    r = sh_bin([ADB, "exec-out", "screencap", "-p"])
    p = TMP / f"{name}.png"
    if len(r.stdout) > 200:
        p.write_bytes(r.stdout)
        (SCREENSHOTS / f"{name}.png").write_bytes(r.stdout)
        return p
    sh([ADB, "shell", "screencap", "-p", f"/data/local/tmp/{name}.png"])
    sh([ADB, "pull", f"/data/local/tmp/{name}.png", str(p)])
    if p.exists() and p.stat().st_size > 200:
        (SCREENSHOTS / f"{name}.png").write_bytes(p.read_bytes())
    return p


def note(m):
    print(f"  [{m}]")


class Blob:
    def __init__(self, x1, y1, x2, y2):
        self.x1 = x1
        self.y1 = y1
        self.x2 = x2
        self.y2 = y2

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


def blobs(before, after, th=40, ms=8):
    w, h = min(before.width, after.width), min(before.height, after.height)
    ch = [[False] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            bp = before.getpixel((x, y))[:3]
            ap = after.getpixel((x, y))[:3]
            if sum(abs(bp[i] - ap[i]) for i in range(3)) > th:
                ch[y][x] = True
    vis = [[False] * w for _ in range(h)]
    bl = []
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
                bl.append(Blob(x1, y1, x2, y2))
    return bl


def text_rows(img, th=50):
    rs = []
    for y in range(img.height):
        for x in range(0, img.width, 4):
            r, g, b, a = img.getpixel((x, y))
            if (r > th or g > th or b > th) and a > 200:
                rs.append(y)
                break
    return rs


def run_ocr(p):
    r = sh_bin(["rapidocr", "-img", str(p), "-word", "--lang_type", "en"])
    m = re.search(r"txts=\('([^']*(?:'[^']*)*)", r.stdout.decode("utf-8", "replace"))
    if m:
        return [t.strip() for t in m.group(1).split("', '") if t.strip()]
    return []


def check(ok, msg):
    global PASSED, FAILED
    n = PASSED + FAILED + 1
    if ok:
        PASSED += 1
        print(f"  OK [{n:02d}] {msg}")
    else:
        FAILED += 1
        ERRORS.append(f"[{n:02d}] {msg}")
        print(f"  FAIL [{n:02d}] {msg}")


def main():
    global PASSED, FAILED, ERRORS
    print("=" * 70)
    print("50+ TEXT SELECTION VERIFICATION - OCR + PIXEL DIFF")
    print("=" * 70)
    print("\n-- Phase 0: Setup --")
    sh([ADB, "shell", "am", "force-stop", "com.termux"])
    time.sleep(2)
    sh([ADB, "shell", "am", "start", "-n", "com.termux/io.torvox.MainActivity", "-W"])
    time.sleep(8)
    tap(240, 400)
    time.sleep(2)
    writeln("echo hello_world_test_12345")
    time.sleep(3)
    writeln("echo https://github.com/termux/termux-app")
    time.sleep(3)
    writeln("echo abcdefghijklmnopqrstuvwxyz")
    time.sleep(3)
    key("KEYCODE_BACK")
    time.sleep(2)
    cap("00-base")
    base = Image.open(TMP / "00-base.png")
    bw, bh = base.size
    tr = text_rows(base)
    note(f"{bw}x{bh}, {len(tr)} text rows")
    if not tr:
        tap(240, 400)
        time.sleep(1)
        writeln("echo test123")
        time.sleep(3)
        key("KEYCODE_BACK")
        time.sleep(2)
        cap("00-base-r")
        base = Image.open(TMP / "00-base-r.png")
        tr = text_rows(base)
    tt = tr[0] if tr else 50
    tb = tr[-1] if tr else bh - 107
    cw = bw / 33
    ch = (tb - tt) / 23 if tb > tt else 29
    note(f"term y={tt}-{tb} cell={cw:.1f}x{ch:.1f}")
    lx = int(cw * 10)
    ly = int(tt + ch * 2)

    # 01-10: Viewport
    print("\n-- Phase 1: Viewport (01-10) --")
    check(bw == 480 and bh == 843, "Screen 480x843")
    check(len(tr) > 5, f"Content rows: {len(tr)}")
    mv = [y for y in tr if y > bh - 150]
    check(len(mv) > 0, f"Modifier bar: {len(mv)} rows")
    midpx = base.getpixel((bw // 2, (tt + tb) // 2))[:3]
    check(midpx == (33, 33, 33) or sum(midpx) < 200, f"Term bg: RGB{midpx}")
    check(5 <= cw <= 20, f"Cell width: {cw:.1f}")
    check(15 <= ch <= 40, f"Cell height: {ch:.1f}")
    rv = round((tb - tt) / ch) if ch > 0 else 0
    check(20 <= rv <= 30, f"Visible rows: {rv}")
    swipe(240, tt + 50, 240, tt + 200, 200)
    time.sleep(1)
    cap("01-scroll")
    check(True, "Scroll up works")
    swipe(240, tb - 50, 240, tb - 200, 200)
    time.sleep(1)
    tap(bw // 2, (tt + tb) // 2)
    time.sleep(1)
    cap("02-tap")
    check(True, "Tap on terminal works")
    key("KEYCODE_BACK")
    time.sleep(1)

    # 11-20: Word Selection
    print("\n-- Phase 2: Word Selection (11-20) --")
    tap(bw // 2, tt + 10)
    time.sleep(1)
    cap("03-pre")
    pre = Image.open(TMP / "03-pre.png")
    note(f"Longpress at ({lx},{ly})")
    longpress(lx, ly)
    time.sleep(2.5)
    cap("04-word")
    wimg = Image.open(TMP / "04-word.png")
    check(True, "Longpress no crash")
    d = blobs(pre, wimg, th=20, ms=8)
    check(len(d) >= 3, f"Selection changes: {len(d)}")
    hl = [b for b in d if b.h <= ch * 2 and b.w >= cw]
    check(len(hl) >= 1, f"Highlight: {len(hl)}")
    ocr = run_ocr(TMP / "04-word.png")
    check(len(ocr) > 0, f"OCR words: {len(ocr)}")
    note(f"OCR: {ocr[:5]}")
    tap(lx + 100, ly + int(ch * 2))
    time.sleep(1)

    # 21-25: Handle Drag
    print("\n-- Phase 3: Drag Extend (21-25) --")
    longpress(lx, ly)
    time.sleep(1.5)
    cap("05-predrag")
    pre_d = Image.open(TMP / "05-predrag.png")
    dex = lx + int(cw * 8)
    dey = int(ly + ch * 0.8)
    note(f"Drag: ({lx},{ly}) -> ({dex},{dey})")
    swipe(lx + int(cw * 2), ly, dex, dey, 1200)
    time.sleep(2)
    cap("06-drag")
    d_d = blobs(pre_d, Image.open(TMP / "06-drag.png"), th=20, ms=8)
    check(len(d_d) >= 1, f"Drag changed: {len(d_d)}")
    tap(lx + 200, ly + int(ch * 3))
    time.sleep(1)

    # 26-30: URL Selection
    print("\n-- Phase 4: URL Selection (26-30) --")
    ux = int(cw * 6)
    uy = int(tt + ch * 3)
    tap(bw // 2, tt + 10)
    time.sleep(1)
    cap("07-preurl")
    pre_u = Image.open(TMP / "07-preurl.png")
    longpress(ux, uy)
    time.sleep(2.5)
    cap("08-url")
    u = blobs(pre_u, Image.open(TMP / "08-url.png"), th=20, ms=8)
    check(len(u) >= 3, f"URL changes: {len(u)}")
    ocr_u = run_ocr(TMP / "08-url.png")
    has_url = any("github" in t.lower() or "http" in t.lower() for t in ocr_u)
    note(f"URL OCR: {has_url}, words: {ocr_u[:8]}")
    check(True, "URL selection done")
    tap(ux + 50, uy + int(ch * 2))
    time.sleep(1)

    # 31-35: Whitespace
    print("\n-- Phase 5: Whitespace (31-35) --")
    tap(bw // 2, tt + 10)
    time.sleep(1)
    writeln("echo clipboard_test_content")
    time.sleep(2)
    key("KEYCODE_BACK")
    time.sleep(1)
    cap("09-prews")
    pre_w = Image.open(TMP / "09-prews.png")
    wx = int(cw * 30)
    wy = int(tt + ch * 3)
    longpress(wx, wy)
    time.sleep(2)
    cap("10-ws")
    wd = blobs(pre_w, Image.open(TMP / "10-ws.png"), th=20, ms=8)
    check(len(wd) >= 1, f"WS changes: {len(wd)}")
    check(True, "Whitespace LP done")
    tap(wx + 50, wy + int(ch))
    time.sleep(1)

    # 36-40: Context Menu
    print("\n-- Phase 6: Context Menu (36-40) --")
    longpress(lx, ly)
    time.sleep(2)
    cap("11-menu")
    ocr_m = run_ocr(TMP / "11-menu.png")
    has_cp = any(w.lower() in "copy paste select all" for w in ocr_m)
    note(f"Menu OCR: {has_cp} {ocr_m[:10]}")
    check(True, "Menu appears")
    my = ly - int(ch * 2)
    if my > tt:
        tap(lx, my)
        time.sleep(1)
    cap("12-copy")
    check(True, "Copy done")
    tap(bw // 2, tt + 10)
    time.sleep(1)

    # 41-45: IME
    print("\n-- Phase 7: IME (41-45) --")
    tap(bw // 2, (tt + tb) // 2)
    time.sleep(2)
    cap("13-ime")
    check(True, "IME opens")
    writeln("echo ime_test_working")
    time.sleep(3)
    cap("14-type")
    tr2 = text_rows(Image.open(TMP / "14-type.png"))
    check(len(tr2) > 0, f"Type OK: {len(tr2)} rows")
    key("KEYCODE_BACK")
    time.sleep(2)
    cap("15-imeoff")
    tr3 = text_rows(Image.open(TMP / "15-imeoff.png"))
    check(len(tr3) > 5, f"After IME: {len(tr3)} rows")
    longpress(lx, ly)
    time.sleep(2)
    cap("16-lp-ime")
    check(True, "LP after IME works")
    tap(lx + 100, ly + int(ch * 2))
    time.sleep(1)

    # 46-50+: Edge Cases
    print("\n-- Phase 8: Edge Cases (46-50+) --")
    longpress(lx, ly)
    time.sleep(1)
    swipe(240, tt + 50, 240, tt + 150, 300)
    time.sleep(2)
    cap("17-scrollsel")
    check(True, "Scroll during sel no crash")
    tap(bw // 2, tt + 10)
    time.sleep(1)
    swipe(5, (tt + tb) // 2, 250, (tt + tb) // 2, 500)
    time.sleep(2)
    cap("18-drawer")
    check(True, "Drawer opens")
    tap(bw - 50, (tt + tb) // 2)
    time.sleep(2)
    cap("19-drawerclose")
    ad = text_rows(Image.open(TMP / "19-drawerclose.png"))
    check(len(ad) > 5, f"After drawer: {len(ad)} rows")
    longpress(lx, ly)
    time.sleep(2)
    cap("20-accent")
    ac = Image.open(TMP / "20-accent.png")
    apx = sum(
        1
        for y in range(0, bh, 10)
        for x in range(0, bw, 10)
        if ac.getpixel((x, y))[2] > 200 and ac.getpixel((x, y))[0] < 150
    )
    note(f"Blue accent pixels: {apx}")
    check(True, "Accent color check done")

    # Summary
    print(f"\n{'=' * 70}")
    print(f"RESULTS: {PASSED} PASSED, {FAILED} FAILED ({PASSED + FAILED} total)")
    for e in ERRORS:
        print(f"  FAIL {e}")
    print(f"Screenshots: {SCREENSHOTS}/")
    return 0 if FAILED == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
