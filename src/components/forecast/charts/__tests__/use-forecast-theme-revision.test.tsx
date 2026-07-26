/* @vitest-environment jsdom */
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { useForecastThemeRevision } from "../use-forecast-theme-revision";

const root = document.documentElement;
const initialTheme = root.getAttribute("data-theme");
const initialPalette = root.getAttribute("data-palette");

afterEach(() => {
  cleanup();
  if (initialTheme == null) root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", initialTheme);
  if (initialPalette == null) root.removeAttribute("data-palette");
  else root.setAttribute("data-palette", initialPalette);
});

describe("useForecastThemeRevision", () => {
  it("signale un changement de theme ou de palette aux canvas", async () => {
    const { result } = renderHook(() => useForecastThemeRevision());
    const initialRevision = result.current;

    act(() => {
      root.setAttribute("data-theme", "forecast-test-theme");
      root.setAttribute("data-palette", "forecast-test-palette");
    });

    await waitFor(() => expect(result.current).toBeGreaterThan(initialRevision));
  });
});
