import assert from "node:assert/strict";

// Shared native journey baseline: hosted macOS screens can cap content below 760px.
const STANDARD_VIEWPORT = { width: 900, height: 600 };

export async function setMinimumViewport(
  width = STANDARD_VIEWPORT.width,
  height = STANDARD_VIEWPORT.height,
): Promise<void> {
  let requestedWidth = width;
  let requestedHeight = height;
  let measured = { width: 0, height: 0 };
  // Drivers differ between physical and logical pixels on Retina displays.
  // Check the rendered viewport instead of accepting a successful resize alone.
  for (let attempt = 0; attempt < 3; attempt += 1) {
    await browser.setWindowSize(requestedWidth, requestedHeight);
    const viewport = await browser.execute(() => ({
      width: window.innerWidth,
      height: window.innerHeight,
    }));
    measured = viewport;
    assert.ok(viewport.width > 0 && viewport.height > 0);
    if (viewport.width >= width && viewport.height >= height) return;
    requestedWidth = Math.ceil(requestedWidth * Math.max(1, width / viewport.width));
    requestedHeight = Math.ceil(requestedHeight * Math.max(1, height / viewport.height));
  }
  assert.fail(`Native viewport did not reach ${width}×${height}; measured ${measured.width}×${measured.height}`);
}
