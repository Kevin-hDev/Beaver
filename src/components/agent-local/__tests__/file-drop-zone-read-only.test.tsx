import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { FileDropZone } from "../file-drop-zone";

const webview = vi.hoisted(() => {
  const listeners = [vi.fn(), vi.fn()];
  let subscriptionCount = 0;
  return {
    listeners,
    unlisten: listeners[0],
    onDragDropEvent: vi.fn(() => Promise.resolve(listeners[subscriptionCount++])),
    reset: () => { subscriptionCount = 0; },
  };
});

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => webview,
}));

afterEach(() => {
  cleanup();
  webview.unlisten.mockClear();
  webview.onDragDropEvent.mockClear();
  webview.listeners[1].mockClear();
  webview.reset();
});

describe("FileDropZone child read-only mode", () => {
  it("does not register a global listener while disabled", () => {
    const props = { dragging: false, onDragChange: vi.fn(), onDropPaths: vi.fn() };
    const { rerender } = render(<FileDropZone {...props} enabled={false}>content</FileDropZone>);

    expect(webview.onDragDropEvent).not.toHaveBeenCalled();

    rerender(<FileDropZone {...props} enabled>content</FileDropZone>);

    expect(webview.onDragDropEvent).toHaveBeenCalledTimes(1);
  });

  it("retire l'abonnement Tauri quand la surface devient inactive", async () => {
    const props = { enabled: true, dragging: false, onDragChange: vi.fn(), onDropPaths: vi.fn() };
    let active = true;
    const view = render(
      <AppSurfaceActivityProvider active={active}>
        <FileDropZone {...props}>content</FileDropZone>
      </AppSurfaceActivityProvider>,
    );
    expect(webview.onDragDropEvent).toHaveBeenCalledTimes(1);

    active = false;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <FileDropZone {...props}>content</FileDropZone>
      </AppSurfaceActivityProvider>,
    );
    await waitFor(() => expect(webview.unlisten).toHaveBeenCalledOnce());
    expect(webview.onDragDropEvent).toHaveBeenCalledTimes(1);

    active = true;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <FileDropZone {...props}>content</FileDropZone>
      </AppSurfaceActivityProvider>,
    );
    await waitFor(() => expect(webview.onDragDropEvent).toHaveBeenCalledTimes(2));
    expect(webview.listeners[1]).not.toHaveBeenCalled();
  });
});
