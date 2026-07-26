import { invoke } from "@tauri-apps/api/core";
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useLocalSites } from "../use-local-sites";

const SCAN = {
  sites: [{
    url: "http://localhost:3000/",
    title: "Application locale",
    port: 3000,
    protocol: "http",
  }],
  generation: 1,
  changed: true,
};

describe("useLocalSites", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.mocked(invoke).mockResolvedValue(SCAN);
  });

  afterEach(() => vi.useRealTimers());

  it("scanne immédiatement puis toutes les cinq secondes seulement sur l'accueil", async () => {
    const { result, rerender } = renderHook(
      ({ visible }) => useLocalSites(visible),
      { initialProps: { visible: true } },
    );
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(result.current.sites).toHaveLength(1);
    expect(invoke).toHaveBeenCalledTimes(1);

    const stableSites = result.current.sites;
    await act(async () => {
      vi.advanceTimersByTime(5_000);
      await Promise.resolve();
    });
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(result.current.sites).toBe(stableSites);

    rerender({ visible: false });
    act(() => { vi.advanceTimersByTime(10_000); });
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
