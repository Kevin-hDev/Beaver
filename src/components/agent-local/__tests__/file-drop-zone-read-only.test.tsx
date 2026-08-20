import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileDropZone } from "../file-drop-zone";

const webview = vi.hoisted(() => ({
  onDragDropEvent: vi.fn(() => Promise.resolve(vi.fn())),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => webview,
}));

describe("FileDropZone child read-only mode", () => {
  it("does not register a global listener while disabled", () => {
    const props = { dragging: false, onDragChange: vi.fn(), onDropPaths: vi.fn() };
    const { rerender } = render(<FileDropZone {...props} enabled={false}>content</FileDropZone>);

    expect(webview.onDragDropEvent).not.toHaveBeenCalled();

    rerender(<FileDropZone {...props} enabled>content</FileDropZone>);

    expect(webview.onDragDropEvent).toHaveBeenCalledTimes(1);
  });
});
