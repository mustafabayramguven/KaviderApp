#!/usr/bin/env python3
"""Generates the M0 zoom-test image.

The pinch/pan/double-tap spike needs one large, visually judgeable image. It is generated
here (stdlib only, no downloads) so the fixture is reproducible and reviewable, rather
than a binary of unknown provenance.

Usage:
    python3 tools/gen-test-image.py            # writes crates/kavider-app/ui/assets/zoom-test.png

The image is deliberately 1600x2400 and uncompressed-in-memory: at 1x it is a 15 MB
ARGB8888 bitmap, which is exactly the case the real reader pipeline (downscale, bounded
LRU, off-thread decode) has to survive in M4.
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

WIDTH = 1600
HEIGHT = 2400

BACKGROUND = (238, 240, 244)
GRID = (198, 204, 214)
MAJOR_GRID = (120, 130, 150)
ACCENT = (120, 40, 200)
CORNER = (200, 60, 60)
DIAGONAL = (30, 120, 90)


def pixel(x: int, y: int) -> tuple[int, int, int]:
    """A deterministic pattern: fine grid, coarse grid, diagonal, corners, centre cross."""
    on_minor_grid = x % 100 == 0 or y % 100 == 0
    on_major_grid = x % 500 == 0 or y % 500 == 0
    on_diagonal = abs((x * HEIGHT) - (y * WIDTH)) < 3000

    # 200x200 corner blocks make zoom and pan unmistakable.
    if (x < 200 or x >= WIDTH - 200) and (y < 200 or y >= HEIGHT - 200):
        return CORNER

    cx, cy = WIDTH // 2, HEIGHT // 2
    if abs(x - cx) < 6 or abs(y - cy) < 6:
        return ACCENT

    if on_diagonal:
        return DIAGONAL
    if on_major_grid:
        return MAJOR_GRID
    if on_minor_grid:
        return GRID
    return BACKGROUND


def write_png(path: Path, width: int, height: int) -> None:
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter type 0 (None)
        for x in range(width):
            raw.extend(pixel(x, y))
    compressed = zlib.compress(bytes(raw), 6)

    def chunk(tag: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)  # 8-bit RGB
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as handle:
        handle.write(b"\x89PNG\r\n\x1a\n")
        handle.write(chunk(b"IHDR", ihdr))
        handle.write(chunk(b"IDAT", compressed))
        handle.write(chunk(b"IEND", b""))


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    target = repo_root / "crates" / "kavider-app" / "ui" / "assets" / "zoom-test.png"
    write_png(target, WIDTH, HEIGHT)
    size = target.stat().st_size
    print(f"wrote {target} ({WIDTH}x{HEIGHT}, {size / 1024:.0f} KiB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
