import assert from "node:assert/strict";

export async function setMinimumViewport(width: number, height: number): Promise<void> {
  let requestedWidth = width;
  let requestedHeight = height;
  // Drivers differ between physical and logical pixels on Retina displays.
  // Check the rendered viewport instead of accepting a successful resize alone.
  for (let attempt = 0; attempt < 3; attempt += 1) {
    await browser.setWindowSize(requestedWidth, requestedHeight);
    const viewport = await browser.execute(() => ({
      width: window.innerWidth,
      height: window.innerHeight,
    }));
    assert.ok(viewport.width > 0 && viewport.height > 0);
    if (viewport.width >= width && viewport.height >= height) return;
    requestedWidth = Math.ceil(requestedWidth * Math.max(1, width / viewport.width));
    requestedHeight = Math.ceil(requestedHeight * Math.max(1, height / viewport.height));
  }
  assert.fail(`Native viewport did not reach ${width}×${height}`);
}
