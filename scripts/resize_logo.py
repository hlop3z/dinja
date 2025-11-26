#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "Pillow>=10.0.0",
# ]
# ///
"""
Resize the Dinja logo to a more GitHub and docs friendly size.

This script resizes the logo to a maximum width of 200px while maintaining
aspect ratio, which is ideal for display in README files and documentation.
"""
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("error: Pillow is required. Install it with: uv run resize_logo.py", file=sys.stderr)
    sys.exit(1)

# Maximum width for the resized logo (good for GitHub README and docs)
MAX_WIDTH = 200

ROOT = Path(__file__).resolve().parent.parent
LOGO_PATH = ROOT / "docs" / "docs" / "assets" / "logo.png"


def resize_logo(max_width: int = MAX_WIDTH) -> None:
    """Resize the logo to a more friendly size for GitHub and documentation."""
    if not LOGO_PATH.exists():
        print(f"error: Logo not found at {LOGO_PATH}", file=sys.stderr)
        sys.exit(1)

    # Open the image
    try:
        img = Image.open(LOGO_PATH)
    except Exception as e:
        print(f"error: Failed to open image: {e}", file=sys.stderr)
        sys.exit(1)

    # Get original dimensions
    original_width, original_height = img.size
    print(f"Original size: {original_width}x{original_height}")

    # Calculate new dimensions maintaining aspect ratio
    if original_width <= max_width:
        print(f"Logo is already {original_width}px wide (≤ {max_width}px), no resize needed.")
        return

    aspect_ratio = original_height / original_width
    new_width = max_width
    new_height = int(new_width * aspect_ratio)

    # Resize the image using high-quality resampling
    resized_img = img.resize((new_width, new_height), Image.Resampling.LANCZOS)

    # Save the resized image (overwrite original)
    resized_img.save(LOGO_PATH, optimize=True)
    print(f"Resized logo to {new_width}x{new_height}")
    print(f"Saved to {LOGO_PATH}")


if __name__ == "__main__":
    resize_logo()

