import { beforeEach, describe, expect, it, vi } from "vitest";
import { isAllowedForecastDevSource, openForecastDevSource } from "../forecast-dev-source";

const { open } = vi.hoisted(() => ({
  open: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open }));

describe("forecast dev source", () => {
  beforeEach(() => open.mockClear());

  it("autorise uniquement les sources officielles en HTTPS", () => {
    expect(isAllowedForecastDevSource("https://pypi.org/project/timesfm/")).toBe(true);
    expect(isAllowedForecastDevSource("https://huggingface.co/amazon/chronos-2")).toBe(true);
    expect(isAllowedForecastDevSource("http://github.com/org/repo")).toBe(false);
    expect(isAllowedForecastDevSource("https://github.com.evil.invalid/org/repo")).toBe(false);
  });

  it("n'ouvre jamais une URL non approuvée", async () => {
    await openForecastDevSource("https://example.invalid/package");
    expect(open).not.toHaveBeenCalled();
    await openForecastDevSource("https://github.com/org/repo");
    expect(open).toHaveBeenCalledOnce();
  });
});
