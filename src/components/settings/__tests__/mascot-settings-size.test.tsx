/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import {
  invokeCalls,
  resetSettingsTestEnvironment,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

describe("taille de la mascotte", () => {
  beforeEach(() => {
    resetSettingsTestEnvironment();
  });

  it("affiche la taille immédiatement mais ne sauvegarde que la dernière valeur", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.mascot"))[0]);

    const slider = await screen.findByLabelText("settings.mascot.sizeTitle");
    for (const value of [105, 110, 115, 120, 125]) {
      fireEvent.change(slider, { target: { value: String(value) } });
    }

    expect(screen.getByText("125%")).toBeTruthy();
    await waitFor(() => {
      const sizeUpdates = invokeCalls().filter(([, args]) => {
        const patch = (args as { patch?: Record<string, unknown> } | undefined)?.patch;
        return typeof patch?.size_percent === "number";
      });
      expect(sizeUpdates).toEqual([
        ["patch_mascot_settings", { patch: { size_percent: 125 } }],
      ]);
    });
  });
});
