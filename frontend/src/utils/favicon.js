// Configuration (matching generate_heatmap.py)
const GRID_SIZE = 17;
const PATTERN_SIZE = 24;
const RECT_SIZE = 1.3;
const SPACING = 1.4;
const STROKE_WIDTH = 0.1;
const CORNER_RADIUS = 0.26;
const COLORS = ['#006d32', '#26a641', '#39d353'];

// GitHub cat path
const CAT_PATH = "M12 1C5.923 1 1 5.923 1 12c0 4.867 3.149 8.979 7.521 10.436.55.096.756-.233.756-.522 0-.262-.013-1.128-.013-2.049-2.764.509-3.479-.674-3.699-1.292-.124-.317-.66-1.293-1.127-1.554-.385-.207-.936-.715-.014-.729.866-.014 1.485.797 1.691 1.128.99 1.663 2.571 1.196 3.204.907.096-.715.385-1.196.701-1.471-2.448-.275-5.005-1.224-5.005-5.432 0-1.196.426-2.186 1.128-2.956-.111-.275-.496-1.402.11-2.915 0 0 .921-.288 3.024 1.128a10.193 10.193 0 0 1 2.75-.371c.936 0 1.871.123 2.75.371 2.104-1.43 3.025-1.128 3.025-1.128.605 1.513.221 2.64.111 2.915.701.77 1.127 1.747 1.127 2.956 0 4.222-2.571 5.157-5.019 5.432.399.344.743 1.004.743 2.035 0 1.471-.014 2.654-.014 3.025 0 .289.206.632.756.522C19.851 20.979 23 16.854 23 12c0-6.077-4.922-11-11-11Z";

/**
 * Generate a random heatmap pattern and update the favicon
 * @param {string} theme - 'dark' or 'light'
 */
export function updateFavicon(theme) {
    const strokeColor = theme === 'dark' ? '#fff' : '#000';

    // Generate random pattern rects
    let patternContent = `<rect width="${PATTERN_SIZE}" height="${PATTERN_SIZE}" fill="${strokeColor}" />`;

    for (let row = 0; row < GRID_SIZE; row++) {
        const yPos = row * SPACING;
        for (let col = 0; col < GRID_SIZE; col++) {
            const xPos = col * SPACING;
            const color = COLORS[Math.floor(Math.random() * COLORS.length)];
            patternContent += `<rect width="${RECT_SIZE}" height="${RECT_SIZE}" x="${xPos}" y="${yPos}" fill="${color}" rx="${CORNER_RADIUS}" stroke="${strokeColor}" stroke-width="${STROKE_WIDTH}" />`;
        }
    }

    // Create the complete SVG
    const svg = `<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <pattern id="heatmap" x="0" y="0" width="${PATTERN_SIZE}" height="${PATTERN_SIZE}" patternUnits="userSpaceOnUse">
      ${patternContent}
    </pattern>
  </defs>
  <path d="${CAT_PATH}" fill="url(#heatmap)" />
  <path d="${CAT_PATH}" fill="none" stroke="${strokeColor}" stroke-width="0.2" />
</svg>`;

    // Convert SVG to data URL
    const dataUrl = `data:image/svg+xml,${encodeURIComponent(svg)}`;

    // Update favicon
    let link = document.querySelector("link[rel~='icon']");
    if (!link) {
        link = document.createElement('link');
        link.rel = 'icon';
        document.head.appendChild(link);
    }
    link.href = dataUrl;
}
