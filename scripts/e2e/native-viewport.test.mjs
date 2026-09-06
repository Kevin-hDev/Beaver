import assert from "node:assert/strict";
import test from "node:test";
import { setMinimumViewport } from "../../tests/e2e/native-viewport.ts";

test("native journeys fit a host that caps content height at 677 pixels", async () => {
  const previous = globalThis.browser;
  let viewport;
  globalThis.browser = {
    async setWindowSize(width, height) { viewport = { width, height: Math.min(height, 677) }; },
    async execute() { return viewport; },
  };
  try {
    await setMinimumViewport();
    assert.deepEqual(viewport, { width: 900, height: 600 });
    // An explicitly required larger viewport still fails, with actual dimensions.
    await assert.rejects(setMinimumViewport(1100, 760), /measured 1100×677/);
  } finally {
    globalThis.browser = previous;
  }
});

test("native viewport corrects driver scaling and checks rendered dimensions", async () => {
  const previous = globalThis.browser;
  let viewport;
  globalThis.browser = {
    async setWindowSize(width, height) { viewport = { width: width / 2, height: height / 2 }; },
    async execute() { return viewport; },
  };
  try {
    await setMinimumViewport();
    assert.deepEqual(viewport, { width: 900, height: 600 });
  } finally {
    globalThis.browser = previous;
  }
});
