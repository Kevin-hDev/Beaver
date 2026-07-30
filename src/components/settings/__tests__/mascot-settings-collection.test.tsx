/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  invokeCalls,
  resetSettingsTestEnvironment,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

describe("collection des mascottes", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    resetSettingsTestEnvironment();
  });

  it.each([
    ["Kova", "kova", "settings.mascot.kovaName"],
    ["Nival", "nival", "settings.mascot.nivalName"],
    ["Mokai", "mokai", "settings.mascot.mokaiName"],
    ["Volt", "volt", "settings.mascot.voltName"],
    ["Raku", "raku", "settings.mascot.rakuName"],
    ["Pico", "pico", "settings.mascot.picoName"],
  ])("sélectionne %s depuis une carte sans liste déroulante", async (
    _name,
    mascotId,
    nameKey,
  ) => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.mascot"))[0]);

    const mascotCard = (await screen.findByText(nameKey)).closest("button");
    expect(mascotCard).toHaveAttribute("aria-pressed", "false");
    expect(screen.queryByRole("combobox")).toBeNull();
    fireEvent.click(mascotCard!);

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "patch_mascot_settings",
        { patch: { mascot_id: mascotId } },
      ]);
      expect(mascotCard).toHaveAttribute("aria-pressed", "true");
      expect(
        document.querySelector(`.msp-bubble [data-mascot-id='${mascotId}']`),
      ).toBeTruthy();
    });
  });
});
