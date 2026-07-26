import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { acquireBrowserNativeOcclusion } from "../browser-native-occlusion";
import { BROWSER_ENGINE_STOPPED_EVENT } from "../browser-events";
import { useBrowserSurface } from "../use-browser-surface";

const TAB_ID = "0123456789abcdef0123456789abcdef";
const eventHandlers = new Map<string, (event: { payload: unknown }) => void>();

class TestResizeObserver {
  observe() {}
  disconnect() {}
}

interface HarnessProps {
  active: boolean;
  url: string | null;
}

interface SurfaceCallArgs {
  request: {
    conversationId: string;
    tabId: string;
    url: string | null;
    bounds: { x: number; y: number; width: number; height: number; visible: boolean };
  };
}

function surfaceRequest(index: number): SurfaceCallArgs["request"] | undefined {
  return (vi.mocked(invoke).mock.calls[index]?.[1] as SurfaceCallArgs | undefined)?.request;
}

function Harness({ active, url }: HarnessProps) {
  const { hostRef, retrySurface } = useBrowserSurface({
    active,
    conversationId: "session-test",
    tabId: TAB_ID,
    url,
    onError: vi.fn(),
  });
  return (
    <div className="asp-panel">
      <div className="asp-slide-wrapper">
        <div data-testid="surface" ref={hostRef} />
        <button type="button" onClick={retrySurface}>retry</button>
      </div>
    </div>
  );
}

describe("useBrowserSurface", () => {
  beforeEach(() => {
    eventHandlers.clear();
    vi.mocked(listen).mockReset().mockImplementation((event, callback) => {
      eventHandlers.set(String(event), callback as (value: { payload: unknown }) => void);
      return Promise.resolve(() => eventHandlers.delete(String(event)));
    });
    vi.stubGlobal("ResizeObserver", TestResizeObserver);
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
      queueMicrotask(() => callback(1));
      return 1;
    });
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
      x: 420,
      y: 180,
      width: 600,
      height: 500,
      top: 180,
      right: 1020,
      bottom: 680,
      left: 420,
      toJSON: () => ({}),
    });
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
  });

  it("place la vue CEF dans la zone exacte et ignore une mesure identique", async () => {
    const { rerender } = render(<Harness active url="https://example.com/" />);
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    expect(surfaceRequest(0)).toMatchObject({
      conversationId: "session-test",
      tabId: TAB_ID,
      url: "https://example.com/",
      bounds: { x: 420, y: 180, width: 600, height: 500, visible: true },
    });

    rerender(<Harness active url="https://example.com/" />);
    await act(async () => { await Promise.resolve(); });
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("cache la vue une seule fois pendant une superposition native", async () => {
    render(<Harness active url="https://example.com/" />);
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    const release = acquireBrowserNativeOcclusion();
    expect(release).not.toBeNull();
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    expect(surfaceRequest(1)).toMatchObject({
      conversationId: "session-test",
      tabId: TAB_ID,
      url: null,
      bounds: { visible: false },
    });

    release?.();
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(3));
    expect(surfaceRequest(2)).toMatchObject({
      url: "https://example.com/",
      bounds: { visible: true },
    });
  });

  it("bloque les relances après un arrêt natif jusqu’à une action manuelle", async () => {
    render(<Harness active url="https://example.com/" />);
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    const stopped = eventHandlers.get(BROWSER_ENGINE_STOPPED_EVENT);
    expect(stopped).toBeDefined();

    act(() => {
      stopped?.({
        payload: {
          eventVersion: 1,
          generation: 1,
          conversationId: "session-test",
          tabId: TAB_ID,
        },
      });
      window.dispatchEvent(new Event("resize"));
    });
    await act(async () => { await Promise.resolve(); });
    expect(invoke).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "retry" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    expect(surfaceRequest(1)).toMatchObject({
      url: "https://example.com/",
      bounds: { visible: true },
    });
  });
});
