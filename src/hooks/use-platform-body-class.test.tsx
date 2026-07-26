import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { IS_MAC } from "@/lib/platform";
import { usePlatformBodyClass } from "./use-platform-body-class";

afterEach(() => {
  document.body.classList.remove("os-mac", "os-other");
});

describe("usePlatformBodyClass", () => {
  it("applies one platform class to every app window and cleans it up", () => {
    const expected = IS_MAC ? "os-mac" : "os-other";
    const opposite = IS_MAC ? "os-other" : "os-mac";
    document.body.classList.add(opposite);

    const { unmount } = renderHook(() => usePlatformBodyClass());

    expect(document.body.classList.contains(expected)).toBe(true);
    expect(document.body.classList.contains(opposite)).toBe(false);

    unmount();
    expect(document.body.classList.contains(expected)).toBe(false);
  });
});
