#!/usr/bin/env python3
# Usage: python3 tools/make_assets.py assets
"""Generate the small PNG assets the lava_ui_builder examples use.

Pure stdlib (zlib + struct), so it runs anywhere. Shapes are drawn with signed-distance
style tests per pixel and 4x supersampling for soft edges.
"""
import math, os, struct, sys, zlib

OUT = sys.argv[1]

def png(path, w, h, px):
    raw = b''.join(b'\x00' + bytes(v for p in row for v in p) for row in px)
    def chunk(t, d):
        c = struct.pack('>I', len(d)) + t + d
        return c + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
    data = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b''))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, 'wb').write(data)

def render(w, h, shade, ss=4):
    """shade(x, y) -> (r,g,b,a) floats 0..1 at a sample point; supersampled."""
    rows = []
    for j in range(h):
        row = []
        for i in range(w):
            acc = [0.0] * 4
            for sj in range(ss):
                for si in range(ss):
                    r, g, b, a = shade(i + (si + 0.5) / ss, j + (sj + 0.5) / ss)
                    acc[0] += r * a; acc[1] += g * a; acc[2] += b * a; acc[3] += a
            n = ss * ss
            a = acc[3] / n
            if a > 0:
                row.append(tuple(int(max(0, min(255, round(c / acc[3] * 255)))) for c in acc[:3]) + (int(round(a * 255)),))
            else:
                row.append((0, 0, 0, 0))
        rows.append(row)
    return rows

def rounded_rect(x, y, x0, y0, x1, y1, r):
    cx = min(max(x, x0 + r), x1 - r); cy = min(max(y, y0 + r), y1 - r)
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r and x0 <= x <= x1 and y0 <= y <= y1

# ── nine-slice frame: white, so the theme tints it ──────────────────────────────────
def frame(x, y):
    S = 48
    if not rounded_rect(x, y, 0, 0, S, S, 12):
        return (0, 0, 0, 0)
    inner = rounded_rect(x, y, 4, 4, S - 4, S - 4, 9)
    bevel = 1.0 - 0.18 * (y / S)          # lighter on top: reads as raised
    if inner:
        return (bevel, bevel, bevel, 1.0)
    return (0.62, 0.62, 0.62, 1.0)         # the rim, darker after tinting
png(f'{OUT}/ui/frame.png', 48, 48, render(48, 48, frame))

# ── item icons, 64x64 ───────────────────────────────────────────────────────────────
def icon(fn):
    return render(64, 64, fn)

def sword(x, y):
    # blade along the diagonal, guard across it, grip below
    u, v = (x - y) / math.sqrt(2), (x + y) / math.sqrt(2)   # v: along blade
    u2 = (x + y - 64) / math.sqrt(2)                       # across the blade
    w = (x - y) / math.sqrt(2)                              # along: +ve towards top-right
    if abs(u2) < 3.5 and -6 < w < 38: return (0.85, 0.88, 0.95, 1)
    if abs(u2) < 12 and -10 < w <= -6: return (0.80, 0.60, 0.15, 1)
    if abs(u2) < 3 and -26 < w <= -10: return (0.45, 0.28, 0.12, 1)
    if (u2) ** 2 + (w + 28) ** 2 < 16: return (0.80, 0.60, 0.15, 1)
    return (0, 0, 0, 0)
def shield(x, y):
    if 10 < x < 54 and 8 < y < 58:
        cx = 32
        if y < 34 or abs(x - cx) < (58 - y) * 0.95:
            edge = abs(x - cx) > 18 or y < 12
            return (0.80, 0.70, 0.25, 1) if edge else (0.20, 0.40, 0.80, 1)
    return (0, 0, 0, 0)
def potion(x, y):
    if (x - 32) ** 2 + (y - 40) ** 2 < 18 ** 2:
        return (0.80, 0.20, 0.70, 1) if y > 34 else (0.85, 0.85, 0.95, 0.6)
    if 26 < x < 38 and 12 < y < 26: return (0.85, 0.85, 0.95, 0.7)
    if 24 < x < 40 and 8 < y < 13: return (0.55, 0.35, 0.20, 1)
    return (0, 0, 0, 0)
def gem(x, y):
    d = abs(x - 32) / 22 + abs(y - 32) / 26
    if d < 1:
        return (0.30, 0.95, 0.75, 1) if x < 32 else (0.20, 0.70, 0.55, 1)
    return (0, 0, 0, 0)
def apple(x, y):
    if (x - 32) ** 2 + (y - 38) ** 2 < 20 ** 2: return (0.85, 0.15, 0.15, 1)
    if 30 < x < 34 and 10 < y < 20: return (0.40, 0.25, 0.10, 1)
    if (x - 40) ** 2 / 64 + (y - 14) ** 2 / 16 < 1: return (0.25, 0.70, 0.25, 1)
    return (0, 0, 0, 0)
def coin(x, y):
    d = (x - 32) ** 2 + (y - 32) ** 2
    if d < 22 ** 2: return (0.95, 0.80, 0.20, 1) if d < 17 ** 2 else (0.80, 0.60, 0.10, 1)
    return (0, 0, 0, 0)
def torch(x, y):
    if 29 < x < 35 and 26 < y < 58: return (0.50, 0.30, 0.15, 1)
    f = (x - 32) ** 2 / 100 + (y - 20) ** 2 / 196
    if f < 1: return (1.0, 0.75 if f < 0.4 else 0.45, 0.10, 1)
    return (0, 0, 0, 0)
def arrow(x, y):
    u, v = (x - y) / math.sqrt(2), (x + y) / math.sqrt(2)
    if abs(u) < 2 and 20 < v < 80: return (0.60, 0.40, 0.20, 1)
    if 74 < v < 90 and abs(u) < (90 - v) * 0.6: return (0.75, 0.75, 0.80, 1)
    if 18 < v < 30 and abs(u) < 7 and abs(u) > 1.5: return (0.90, 0.90, 0.90, 1)
    return (0, 0, 0, 0)
for name, fn in [('sword', sword), ('shield', shield), ('potion', potion), ('gem', gem),
                 ('apple', apple), ('gold', coin), ('torch', torch), ('arrow', arrow)]:
    png(f'{OUT}/icons/{name}.png', 64, 64, icon(fn))

# ── avatars, 64x64: a face on a coloured disc ───────────────────────────────────────
def avatar(bg, skin, hair):
    def f(x, y):
        if (x - 32) ** 2 + (y - 32) ** 2 > 31 ** 2: return (0, 0, 0, 0)
        if (x - 32) ** 2 + (y - 60) ** 2 < 22 ** 2: return hair + (1,)       # shoulders
        if (x - 32) ** 2 + (y - 28) ** 2 < 13 ** 2:
            if y < 22 and (x - 32) ** 2 + (y - 26) ** 2 > 9 ** 2: return hair + (1,)
            if (abs(x - 27) < 1.6 or abs(x - 37) < 1.6) and abs(y - 29) < 1.6: return (0.1, 0.1, 0.1, 1)
            return skin + (1,)
        return bg + (1,)
    return f
avatars = [((0.30, 0.45, 0.80), (0.95, 0.80, 0.65), (0.25, 0.15, 0.10)),
           ((0.80, 0.35, 0.30), (0.75, 0.55, 0.40), (0.10, 0.10, 0.10)),
           ((0.35, 0.65, 0.40), (0.95, 0.85, 0.75), (0.85, 0.70, 0.30)),
           ((0.60, 0.40, 0.75), (0.55, 0.38, 0.28), (0.15, 0.10, 0.08)),
           ((0.85, 0.65, 0.25), (0.98, 0.88, 0.80), (0.70, 0.30, 0.15)),
           ((0.25, 0.60, 0.70), (0.80, 0.62, 0.48), (0.30, 0.30, 0.32))]
for i, a in enumerate(avatars):
    png(f'{OUT}/avatars/{i}.png', 64, 64, render(64, 64, avatar(*a), ss=3))

# ── minimap, 160x160: terrain blobs, a river, a road, a player arrow ────────────────
def minimap(x, y):
    n = math.sin(x * 0.07) * math.cos(y * 0.06) + 0.5 * math.sin((x + y) * 0.11)
    base = (0.22, 0.42, 0.22) if n > -0.2 else (0.30, 0.52, 0.28)
    if n > 0.8: base = (0.45, 0.42, 0.35)
    if abs(y - (80 + 25 * math.sin(x * 0.05))) < 5: base = (0.20, 0.40, 0.75)
    if abs(x - 100) < 2.5: base = (0.60, 0.55, 0.45)
    if (x - 80) ** 2 + (y - 80) ** 2 < 5 ** 2: base = (1.0, 0.9, 0.2)
    edge = min(x, y, 160 - x, 160 - y)
    if edge < 3: base = (0.08, 0.08, 0.10)
    return base + (1,)
png(f'{OUT}/minimap.png', 160, 160, render(160, 160, minimap, ss=1))

# ── hero picture for the responsive menu, 256x256: a stylised volcano at dusk ───────
def hero(x, y):
    sky = (0.20 + 0.45 * y / 256, 0.12 + 0.10 * y / 256, 0.35 - 0.15 * y / 256)
    if (x - 190) ** 2 + (y - 60) ** 2 < 22 ** 2: sky = (0.98, 0.85, 0.55)
    mountain = 96 + abs(x - 128) * 1.1
    if y > mountain:
        lava = abs(x - 128 + 6 * math.sin(y * 0.15)) < max(2.5, (y - 96) * 0.06) and y < 200
        return ((0.95, 0.45, 0.10) if lava else (0.18, 0.12, 0.14)) + (1,)
    if y > 230: return (0.10, 0.08, 0.10, 1)
    return sky + (1,)
png(f'{OUT}/hero.png', 256, 256, render(256, 256, hero, ss=1))
print('ok')
