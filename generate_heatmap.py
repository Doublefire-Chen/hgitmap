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

# File paths with their stroke colors
SVG_FILES = [
    ("frontend/src/assets/heatmap-dark.svg", "#fff"),  # White gaps for dark theme
    ("frontend/src/assets/heatmap-light.svg", "#000"),  # Black gaps for light theme
]


def generate_heatmap_pattern(stroke_color):
    """Generate the complete heatmap pattern"""
    lines = []
    lines.append(
        f'    <pattern id="heatmap" x="0" y="0" width="{PATTERN_SIZE}" height="{PATTERN_SIZE}" patternUnits="userSpaceOnUse">')

    # Add background rectangle for light theme to make gaps visible
    if stroke_color == "#000":
        lines.append(f'      <!-- Background for gaps -->')
        lines.append(f'      <rect width="{PATTERN_SIZE}" height="{PATTERN_SIZE}" fill="#000" />')
        lines.append("")

    for row in range(GRID_SIZE):
        lines.append(f"      <!-- Row {row + 1} -->")
        y_pos = row * SPACING

        for col in range(GRID_SIZE):
            x_pos = col * SPACING
            color = random.choice(COLORS)

            rect = (f'      <rect width="{RECT_SIZE}" height="{RECT_SIZE}" '
                    f'x="{x_pos}" y="{y_pos}" fill="{color}" '
                    f'rx="{CORNER_RADIUS}" stroke="{stroke_color}" stroke-width="{STROKE_WIDTH}" />')
            lines.append(rect)

        lines.append("")  # Empty line between rows

    lines.append("    </pattern>")

    return "\n".join(lines)


def update_svg_files():
    """Read SVG files, replace pattern section, and write back"""
    for svg_file, stroke_color in SVG_FILES:
        # Read the existing SVG file
        with open(svg_file, 'r') as f:
            svg_content = f.read()

        # Generate new pattern with appropriate stroke color
        new_pattern = generate_heatmap_pattern(stroke_color)

        # Replace the pattern section using regex
        # Match from <pattern ... > to </pattern>
        pattern_regex = r'<pattern id="heatmap"[^>]*>.*?</pattern>'
        updated_svg = re.sub(pattern_regex, new_pattern,
                             svg_content, flags=re.DOTALL)

        # Write back to file
        with open(svg_file, 'w') as f:
            f.write(updated_svg)

        print(f"✓ Updated {svg_file} with {GRID_SIZE}x{GRID_SIZE} heatmap pattern")

    print(f"\nConfiguration:")
    print(f"  - Pattern size: {PATTERN_SIZE}x{PATTERN_SIZE}")
    print(f"  - Rectangle size: {RECT_SIZE}")
    print(f"  - Spacing: {SPACING}")
    print(f"  - Stroke width: {STROKE_WIDTH}")


if __name__ == "__main__":
    update_svg_files()
