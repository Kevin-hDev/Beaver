import { fireEvent, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAppLayoutShortcuts } from "../use-app-layout-effects";

describe("useAppLayoutShortcuts", () => {
  it("ne capte pas Ctrl+Alt+B reserve a la preview", () => {
    const toggleSidebar = vi.fn();

    renderHook(() => useAppLayoutShortcuts({
      onBack: vi.fn(),
      onForward: vi.fn(),
      toggleSearch: vi.fn(),
      toggleSidebar,
    }));

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(toggleSidebar).not.toHaveBeenCalled();
  });
});
