"""生成 ProHelm 占位图标：舵轮（Ship Wheel）图案。

纯标准库实现（无 Pillow 依赖），输出：
- icons/32x32.png
- icons/128x128.png
- icons/128x128@2x.png（256x256）
- icons/icon.ico（内嵌 32x32 BMP）

S3 视觉外包落地后替换为正式 Logo。
"""

from __future__ import annotations

import io
import math
import os
import struct
import zlib

BG = (37, 99, 235)       # blue-600
FG = (255, 255, 255)     # 白色舵轮
CORNER_RADIUS_RATIO = 0.20
OUT_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "apps", "desktop", "src-tauri", "icons",
)


def rounded_rect_sdf(px: float, py: float, size: float) -> float:
    """<=0 在圆角矩形内部。中心为原点坐标系。"""
    half = size / 2
    r = size * CORNER_RADIUS_RATIO
    qx = abs(px) - (half - r)
    qy = abs(py) - (half - r)
    dx = max(qx, 0.0)
    dy = max(qy, 0.0)
    return (dx * dx + dy * dy) ** 0.5 - r


def seg_dist(px, py, x1, y1, x2, y2) -> float:
    vx, vy = x2 - x1, y2 - y1
    wx, wy = px - x1, py - y1
    c1 = vx * wx + vy * wy
    if c1 <= 0:
        return (px - x1) ** 2 + (py - y1) ** 2
    c2 = vx * vx + vy * vy
    if c2 <= c1:
        return (px - x2) ** 2 + (py - y2) ** 2
    t = c1 / c2
    proj_x, proj_y = x1 + t * vx, y1 + t * vy
    return (px - proj_x) ** 2 + (py - proj_y) ** 2


def sample(fx: float, fy: float, size: float) -> tuple[int, int, int, int]:
    """返回该点的覆盖颜色（带简单 alpha 混合边界）。"""
    inside = rounded_rect_sdf(fx, fy, size) <= 0
    if not inside:
        return (0, 0, 0, 0)

    cx = cy = 0.0
    r_out = size * 0.30          # 舵轮外圈半径
    ring_w = size * 0.045
    hub_r = size * 0.055
    spoke_w = size * 0.032
    spoke_len = r_out

    d = (fx - cx) ** 2 + (fy - cy) ** 2
    dist = d ** 0.5
    ring_hit = abs(dist - r_out) <= ring_w / 2
    hub_hit = dist <= hub_r
    spoke_hit = False
    for i in range(8):
        ang = i * math.pi / 4
        x2, y2 = cx + spoke_len * math.cos(ang), cy + spoke_len * math.sin(ang)
        if seg_dist(fx, fy, cx, cy, x2, y2) <= (spoke_w / 2) ** 2:
            spoke_hit = True
            break
    color = FG if (ring_hit or hub_hit or spoke_hit) else BG
    return (*color, 255)


def render(size: int, ss: int = 4) -> list[list[tuple[int, int, int, int]]]:
    px: list[list[tuple[int, int, int, int]]] = []
    step = 1.0 / ss
    for y in range(size):
        row: list[tuple[int, int, int, int]] = []
        for x in range(size):
            r_acc = g_acc = b_acc = a_acc = 0
            for sy in range(ss):
                for sx in range(ss):
                    fx = x - size / 2 + (sx + 0.5) * step
                    fy = y - size / 2 + (sy + 0.5) * step
                    c = sample(fx, fy, size)
                    r_acc += c[0]; g_acc += c[1]; b_acc += c[2]; a_acc += c[3]
            n = ss * ss
            row.append((r_acc // n, g_acc // n, b_acc // n, a_acc // n))
        px.append(row)
    return px


def png_chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png_stream(out: io.BytesIO, px: list[list[tuple[int, int, int, int]]]) -> None:
    h, w = len(px), len(px[0])
    raw = b""
    for row in px:
        raw += b"\x00" + b"".join(bytes(c) for c in row)
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    out.write(
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", zlib.compress(raw, 9))
        + png_chunk(b"IEND", b"")
    )


def write_png(path: str, px: list[list[tuple[int, int, int, int]]]) -> None:
    with open(path, "wb") as f:
        write_png_stream(f, px)


# ICO 内嵌尺寸：覆盖桌面 48、任务栏 24/32、高 DPI（72/96）与资源管理器大图标
ICO_SIZES = [16, 20, 24, 30, 32, 48, 64, 72, 96, 128, 256]


def ico_frame_bmp(px: list[list[tuple[int, int, int, int]]]) -> bytes:
    """单帧 32bpp BMP（BITMAPINFOHEADER + 自底向上 XOR + AND 掩码）。"""
    w = len(px[0])
    h = len(px)
    and_stride = ((w + 31) // 32) * 4
    xor_size = w * h * 4
    and_size = and_stride * h
    bmp = struct.pack(
        "<IiiHHIIiiII", 40, w, h * 2, 1, 32, 0, xor_size + and_size, 0, 0, 0, 0
    )
    xor = b""
    for row in reversed(px):  # BMP 自底向上
        for (r, g, b, a) in row:
            xor += bytes((b, g, r, a))
    return bmp + xor + b"\x00" * and_size


def write_ico(path: str) -> None:
    """多尺寸 **BMP 帧** ICO：每个尺寸从矢量独立渲染（质量优于缩放）。

    - 旧版只嵌 1 张 32x32 BMP，桌面（48px/72px）拉伸显示导致快捷方式图标发糊；
    - 曾改用 PNG 帧（体积小），但 Windows SDK rc.exe 不支持 PNG-in-ICO，
      多帧被"吞"成单帧——故保持全 BMP 帧，兼容 rc.exe / llvm-rc / windres。
    """
    entries: list[tuple[int, int, bytes]] = []
    for size in ICO_SIZES:
        ss = 8 if size <= 32 else (4 if size <= 64 else 2)
        px = render(size, ss=ss)
        entries.append((size, size, ico_frame_bmp(px)))

    header = struct.pack("<HHH", 0, 1, len(entries))
    directory = b""
    offset = 6 + 16 * len(entries)
    for w, h, data in entries:
        directory += struct.pack(
            "<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(data), offset
        )
        offset += len(data)
    blob = header + directory + b"".join(d for _, _, d in entries)
    with open(path, "wb") as f:
        f.write(blob)
    print(f"OK icon.ico ({len(entries)} 尺寸 BMP 帧: {', '.join(map(str, ICO_SIZES))})")


def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)
    for name, size in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)):
        px = render(size)
        write_png(os.path.join(OUT_DIR, name), px)
        print(f"OK {name} ({size}x{size})")
    write_ico(os.path.join(OUT_DIR, "icon.ico"))
    print(f"输出目录: {OUT_DIR}")


if __name__ == "__main__":
    main()
