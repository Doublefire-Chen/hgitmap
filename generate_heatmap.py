#!/usr/bin/env python3
"""
Generate and update SVG heatmap pattern with consistent spacing.
Usage: python generate_heatmap.py
"""

import random
import re

# Configuration
GRID_SIZE = 17  # 17x17 grid to cover 24x24 pattern without repetition
PATTERN_SIZE = 24  # SVG pattern width/height
RECT_SIZE = 1.3  # Size of each rectangle
# Spacing between rectangles (RECT_SIZE + stroke_width for touching edges)
SPACING = 1.4
STROKE_WIDTH = 0.1
CORNER_RADIUS = 0.26

# Color palette (GitHub-style greens)
COLORS = ["#006d32", "#26a641", "#39d353"]
STROKE_COLOR = "#fff"  # White gaps between squares

# File path
SVG_FILE = "frontend/src/assets/heatmap.svg"


def generate_heatmap_pattern():
    """Generate the complete heatmap pattern"""
    lines = []
    lines.append(
        f'    <pattern id="heatmap" x="0" y="0" width="{PATTERN_SIZE}" height="{PATTERN_SIZE}" patternUnits="userSpaceOnUse">')

    for row in range(GRID_SIZE):
        lines.append(f"      <!-- Row {row + 1} -->")
        y_pos = row * SPACING

        for col in range(GRID_SIZE):
            x_pos = col * SPACING
            color = random.choice(COLORS)

            rect = (f'      <rect width="{RECT_SIZE}" height="{RECT_SIZE}" '
                    f'x="{x_pos}" y="{y_pos}" fill="{color}" '
                    f'rx="{CORNER_RADIUS}" stroke="{STROKE_COLOR}" stroke-width="{STROKE_WIDTH}" />')
            lines.append(rect)

        lines.append("")  # Empty line between rows

    lines.append("    </pattern>")

    return "\n".join(lines)


def update_svg_file():
    """Read SVG file, replace pattern section, and write back"""
    # Read the existing SVG file
    with open(SVG_FILE, 'r') as f:
        svg_content = f.read()

    # Generate new pattern
    new_pattern = generate_heatmap_pattern()

    # Replace the pattern section using regex
    # Match from <pattern ... > to </pattern>
    pattern_regex = r'<pattern id="heatmap"[^>]*>.*?</pattern>'
    updated_svg = re.sub(pattern_regex, new_pattern,
                         svg_content, flags=re.DOTALL)

    # Write back to file
    with open(SVG_FILE, 'w') as f:
        f.write(updated_svg)

    print(f"✓ Updated {SVG_FILE} with {GRID_SIZE}x{GRID_SIZE} heatmap pattern")
    print(f"  - Pattern size: {PATTERN_SIZE}x{PATTERN_SIZE}")
    print(f"  - Rectangle size: {RECT_SIZE}")
    print(f"  - Spacing: {SPACING}")
    print(f"  - Stroke width: {STROKE_WIDTH}")


if __name__ == "__main__":
    update_svg_file()
