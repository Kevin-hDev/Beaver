import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  BrowserCapabilityProvider,
  initialBrowserCapability,
  useBrowserCapability,
} from "../use-browser-capability";

function wrapper({ children }: { children: ReactNode }) {
  return <BrowserCapabilityProvider>{children}</BrowserCapabilityProvider>;
}

describe("useBrowserCapability", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockResolvedValue({
      status: "ready",
      engineVersion: "150.0.0+150.0.10",
    });
  });

  it("affiche un état indisponible sur macOS et Windows pendant le contrôle", () => {
    expect(initialBrowserCapability(false)).toEqual({
      status: "unavailable",
      restartRecommended: false,
    });
    expect(initialBrowserCapability(true)).toEqual({ status: "hidden" });
  });

  it("reste visible mais indisponible jusqu'à la confirmation du moteur natif", async () => {
    const { result } = renderHook(() => useBrowserCapability(), { wrapper });

    expect(result.current.status).toBe("unavailable");
    await waitFor(() => expect(result.current.status).toBe("ready"));
    expect(invoke).toHaveBeenCalledWith("browser_capability");
  });

  it("reçoit la disponibilité publiée après le contrôle des cookies", async () => {
    let handler: ((event: { payload: unknown }) => void) | null = null;
    vi.mocked(invoke).mockResolvedValue({
      status: "unavailable",
      restartRecommended: false,
    });
    vi.mocked(listen).mockImplementation((_event, callback) => {
      handler = callback as (event: { payload: unknown }) => void;
      return Promise.resolve(() => {});
    });
    const { result } = renderHook(() => useBrowserCapability(), { wrapper });
    await waitFor(() => expect(result.current.status).toBe("unavailable"));

    act(() => handler?.({
      payload: { status: "ready", engineVersion: "150.0.0+150.0.10" },
    }));

    expect(result.current.status).toBe("ready");
  });
});
