/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  invokeCalls,
  resetSettingsTestEnvironment,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

describe("Kova mascot settings", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    resetSettingsTestEnvironment();
  });

  it("sélectionne Kova depuis une carte sans liste déroulante", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.mascot"))[0]);

    const kovaCard = (await screen.findByText("settings.mascot.kovaName")).closest("button");
    expect(kovaCard).toHaveAttribute("aria-pressed", "false");
    expect(screen.queryByRole("combobox")).toBeNull();
    fireEvent.click(kovaCard!);

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "patch_mascot_settings",
        { patch: { mascot_id: "kova" } },
      ]);
      expect(kovaCard).toHaveAttribute("aria-pressed", "true");
      expect(document.querySelector(".msp-bubble [data-mascot-id='kova']")).toBeTruthy();
    });
  });
});
