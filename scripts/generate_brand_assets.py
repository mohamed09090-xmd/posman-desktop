#!/usr/bin/env python3
"""Normalize the approved POSMAN mark into a transparent square master asset."""
from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


BRAND_RGB = (31, 90, 69)
MASTER_SIZE = 1024
SAFE_AREA = 0.82


def recover_alpha(pixel: tuple[int, int, int, int]) -> int:
    """Recover foreground coverage from a mark composited over opaque white."""
    red, green, blue, source_alpha = pixel
    if source_alpha == 0:
        return 0
    estimates = (
        (255 - red) / (255 - BRAND_RGB[0]),
        (255 - green) / (255 - BRAND_RGB[1]),
        (255 - blue) / (255 - BRAND_RGB[2]),
    )
    coverage = max(0.0, min(1.0, sum(estimates) / len(estimates)))
    # The approved export contains faint near-white compression noise. A
    # deliberate coverage floor removes it without touching the solid mark or
    # its anti-aliased edge.
    if coverage < 0.08:
        return 0
    return round(coverage * source_alpha)


def normalize(source: Path, destination: Path) -> None:
    image = Image.open(source).convert("RGBA")
    pixels = [
        (*BRAND_RGB, recover_alpha(pixel))
        for pixel in image.get_flattened_data()
    ]
    normalized = Image.new("RGBA", image.size)
    normalized.putdata(pixels)
    bounds = normalized.getchannel("A").getbbox()
    if bounds is None:
        raise SystemExit("The source image does not contain a visible mark")

    cropped = normalized.crop(bounds)
    maximum = round(MASTER_SIZE * SAFE_AREA)
    scale = min(maximum / cropped.width, maximum / cropped.height)
    resized = cropped.resize(
        (round(cropped.width * scale), round(cropped.height * scale)),
        Image.Resampling.LANCZOS,
    )
    resized_alpha = resized.getchannel("A")
    resized = Image.new("RGBA", resized.size, (*BRAND_RGB, 0))
    resized.putalpha(resized_alpha)
    master = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    offset = ((MASTER_SIZE - resized.width) // 2, (MASTER_SIZE - resized.height) // 2)
    # Paste the straight-alpha pixels directly. Compositing over transparent
    # black would introduce small RGB rounding shifts in otherwise solid brand
    # pixels.
    master.paste(resized, offset)
    destination.parent.mkdir(parents=True, exist_ok=True)
    master.save(destination, format="PNG", optimize=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("assets/branding/posman-app-icon.png"),
    )
    args = parser.parse_args()
    normalize(args.source, args.output)
    print(f"POSMAN brand master written to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
