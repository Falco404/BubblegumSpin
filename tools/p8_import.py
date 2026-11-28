#!/usr/bin/env python3
"""Extract the sprite sheet (and map) from the upstream cart into assets/*.bin.

The cart itself is gitignored (`assets/upstream/`, "No License"); what this writes
is the derived data the port draws with, in the same spirit as
`tools/extract_level.py` writing `src/consts.rs`.

Usage:
    python3 tools/p8_import.py            # write assets/sprites.bin, assets/map.bin
    python3 tools/p8_import.py --check    # exit 1 if the checked-in files are stale
    python3 tools/p8_import.py --png out/sheet.png   # render the sheet for eyeballing
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CART = os.path.join(ROOT, "assets", "upstream", "bubblegum_spin.p8")

# The PICO-8 system palette, for the optional PNG preview only.
PALETTE = [
    (0x00, 0x00, 0x00), (0x1D, 0x2B, 0x53), (0x7E, 0x25, 0x53), (0x00, 0x87, 0x51),
    (0xAB, 0x52, 0x36), (0x5F, 0x57, 0x4F), (0xC2, 0xC3, 0xC7), (0xFF, 0xF1, 0xE8),
    (0xFF, 0x00, 0x4D), (0xFF, 0xA3, 0x00), (0xFF, 0xEC, 0x27), (0x00, 0xE4, 0x36),
    (0x29, 0xAD, 0xFF), (0x83, 0x76, 0x9C), (0xFF, 0x77, 0xA8), (0xFF, 0xCC, 0xAA),
]


def read_sections(path):
    """Split a .p8 into {section_name: [lines]} (comments and blanks dropped)."""
    sections = {}
    current = None
    with open(path, encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            line = raw.rstrip("\r\n")
            if line.startswith("__") and line.endswith("__"):
                current = line.strip("_")
                sections[current] = []
            elif current is not None:
                sections[current].append(line)
    return sections


def gfx_bytes(sections):
    """The 128x128 sprite sheet, two pixels per byte (low nibble = left pixel)."""
    rows = [r for r in sections["gfx"] if r.strip()]
    if len(rows) != 128:
        raise SystemExit(f"expected 128 gfx rows, found {len(rows)}")
    out = bytearray()
    for row in rows:
        row = row.strip()
        if len(row) != 128:
            raise SystemExit(f"expected 128 nibbles per row, found {len(row)}")
        for i in range(0, 128, 2):
            out.append(int(row[i], 16) | (int(row[i + 1], 16) << 4))
    if len(out) != 8192:
        raise SystemExit(f"expected 8192 bytes, wrote {len(out)}")
    return bytes(out)


def map_bytes(sections):
    """The 128x32 map, one byte per tile."""
    rows = [r for r in sections.get("map", []) if r.strip()]
    out = bytearray()
    for row in rows:
        row = row.strip()
        for i in range(0, len(row), 2):
            out.append(int(row[i:i + 2], 16))
    return bytes(out)


def sfx_bytes(sections):
    """64 sound effects in PICO-8's 68-byte RAM layout.

    The `.p8` text form is *not* the RAM layout: a line is 84 bytes, with four
    header bytes (editor mode, speed, loop start, loop end) followed by 32 notes of
    **20 bits** (five nibbles: pitch hi/lo, waveform, volume, effect). The RAM form
    packs each note into 16 bits, so this converts one to the other.

    RAM note layout (little endian):
      low byte:  w w p p p p p p
      high byte: c e e e v v v w
    """
    rows = [r for r in sections.get("sfx", []) if r.strip()]
    if len(rows) != 64:
        raise SystemExit(f"expected 64 sfx lines, found {len(rows)}")
    out = bytearray()
    for row in rows:
        data = bytes.fromhex(row.strip())
        if len(data) != 84:
            raise SystemExit(f"expected 84 bytes per sfx line, found {len(data)}")
        nibbles = []
        for byte in data[4:]:
            nibbles.append(byte >> 4)
            nibbles.append(byte & 0x0F)
        for n in range(32):
            base = n * 5
            pitch = (nibbles[base] << 4) | nibbles[base + 1]
            wave = nibbles[base + 2]
            volume = nibbles[base + 3]
            effect = nibbles[base + 4]
            if pitch > 63 or volume > 7 or effect > 7 or wave > 15:
                raise SystemExit(f"note {n} out of range in sfx line: {row}")
            lsb = pitch | ((wave & 3) << 6)
            msb = ((wave >> 2) & 1) | (volume << 1) | (effect << 4) | (((wave >> 3) & 1) << 7)
            out.append(lsb)
            out.append(msb)
        # The four header bytes come last, exactly as in PICO-8's RAM layout.
        out.extend(data[0:4])  # mode, speed, loop start, loop end
    if len(out) != 64 * 68:
        raise SystemExit(f"expected 4352 bytes, wrote {len(out)}")
    return bytes(out)


def music_bytes(sections):
    """64 music patterns: four channel bytes each (the sfx id per channel)."""
    rows = [r.strip() for r in sections.get("music", []) if r.strip()]
    if len(rows) != 64:
        raise SystemExit(f"expected 64 music lines, found {len(rows)}")
    out = bytearray()
    for row in rows:
        text = row.replace(" ", "")
        if len(text) != 10:
            raise SystemExit(f"expected 10 hex digits per music line, found {len(text)}")
        out.extend(bytes.fromhex(text[2:]))
    return bytes(out)


def music_flags_bytes(sections):
    """The first two hex digits of each music line: loop-start / loop-end / stop."""
    rows = [r.strip() for r in sections.get("music", []) if r.strip()]
    out = bytearray()
    for row in rows:
        text = row.replace(" ", "")
        out.append(int(text[0:2], 16))
    return bytes(out)


def write_if_changed(path, data, check):
    old = None
    if os.path.exists(path):
        with open(path, "rb") as fh:
            old = fh.read()
    if old == data:
        print(f"{os.path.relpath(path, ROOT)}: up to date ({len(data)} bytes)")
        return True
    if check:
        print(f"{os.path.relpath(path, ROOT)}: STALE ({len(data)} bytes expected)")
        return False
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as fh:
        fh.write(data)
    print(f"{os.path.relpath(path, ROOT)}: wrote {len(data)} bytes")
    return True


def render_png(sheet, path, scale=2):
    from PIL import Image

    img = Image.new("RGB", (128 * scale, 128 * scale))
    for y in range(128):
        for x in range(128):
            byte = sheet[(y * 128 + x) // 2]
            c = (byte & 0x0F) if x % 2 == 0 else (byte >> 4)
            for dy in range(scale):
                for dx in range(scale):
                    img.putpixel((x * scale + dx, y * scale + dy), PALETTE[c])
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    img.save(path)
    print(f"wrote {path}")


def main(argv):
    check = "--check" in argv
    png = None
    if "--png" in argv:
        png = argv[argv.index("--png") + 1]

    sections = read_sections(CART)
    sheet = gfx_bytes(sections)

    if png:
        render_png(sheet, png)

    ok = write_if_changed(os.path.join(ROOT, "assets", "sprites.bin"), sheet, check)
    if "map" in sections:
        ok &= write_if_changed(
            os.path.join(ROOT, "assets", "map.bin"), map_bytes(sections), check
        )
    if "sfx" in sections:
        ok &= write_if_changed(
            os.path.join(ROOT, "assets", "sfx.bin"), sfx_bytes(sections), check
        )
    if "music" in sections:
        ok &= write_if_changed(
            os.path.join(ROOT, "assets", "music.bin"), music_bytes(sections), check
        )
        ok &= write_if_changed(
            os.path.join(ROOT, "assets", "music_flags.bin"),
            music_flags_bytes(sections),
            check,
        )
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
