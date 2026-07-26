import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";

describe("platform constants", () => {
  const originalUA = navigator.userAgent;

  afterEach(() => {
    Object.defineProperty(navigator, "userAgent", {
      value: originalUA,
      configurable: true,
    });
    vi.resetModules();
  });

  describe("on macOS (userAgent contains 'Mac')", () => {
    beforeEach(() => {
      Object.defineProperty(navigator, "userAgent", {
        value: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
        configurable: true,
      });
    });

    it("IS_MAC is true", async () => {
      const { IS_MAC } = await import("../platform");
      expect(IS_MAC).toBe(true);
    });

    it("MOD is ⌘", async () => {
      const { MOD } = await import("../platform");
      expect(MOD).toBe("⌘");
    });

    it("MOD_KEY is metaKey", async () => {
      const { MOD_KEY } = await import("../platform");
      expect(MOD_KEY).toBe("metaKey");
    });

    it("ALT is ⌥", async () => {
      const { ALT } = await import("../platform");
      expect(ALT).toBe("⌥");
    });
  });

  describe("on Windows/Linux (userAgent without 'Mac')", () => {
    beforeEach(() => {
      Object.defineProperty(navigator, "userAgent", {
        value: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        configurable: true,
      });
    });

    it("IS_MAC is false", async () => {
      const { IS_MAC } = await import("../platform");
      expect(IS_MAC).toBe(false);
    });

    it("MOD is Ctrl+", async () => {
      const { MOD } = await import("../platform");
      expect(MOD).toBe("Ctrl+");
    });

    it("MOD_KEY is ctrlKey", async () => {
      const { MOD_KEY } = await import("../platform");
      expect(MOD_KEY).toBe("ctrlKey");
    });

    it("ALT is Alt+", async () => {
      const { ALT } = await import("../platform");
      expect(ALT).toBe("Alt+");
    });
  });
});
