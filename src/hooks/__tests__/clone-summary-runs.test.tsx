import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useCloneSummaryRun } from "../clone-summary-runs";

describe("clone summary subscriptions", () => {
  it("refuse explicitement la saturation et accepte après démontage", () => {
    const views = Array.from({ length: 64 }, (_, index) =>
      renderHook(() => useCloneSummaryRun(`session-${index}`)));

    expect(() => renderHook(() => useCloneSummaryRun("overflow")))
      .toThrow("Active view subscription limit reached");

    views.pop()?.unmount();
    const replacement = renderHook(() => useCloneSummaryRun("replacement"));
    replacement.unmount();
    views.forEach(({ unmount }) => unmount());
  });
});
