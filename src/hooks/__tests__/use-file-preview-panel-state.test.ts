import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useFilePreviewPanelState } from "../use-file-preview-panel-state";

describe("useFilePreviewPanelState", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });

  it("ne relit le stockage qu'une fois tant que la session ne change pas", () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem");
    const { rerender } = renderHook(
      ({ sessionId }) => useFilePreviewPanelState(sessionId),
      { initialProps: { sessionId: "session-a" } },
    );
    const readsAfterMount = getItem.mock.calls.length;

    rerender({ sessionId: "session-a" });
    rerender({ sessionId: "session-a" });

    expect(getItem).toHaveBeenCalledTimes(readsAfterMount);
  });
});
