/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  failInvokeCommand,
  invokeCalls,
  resetSettingsTestEnvironment,
  restoreInvokeCommand,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

async function openMemorySettings() {
  render(<SettingsHarness />);
  fireEvent.click((await screen.findAllByText("settings.tabs.memory"))[0]);
  return {
    mode: await screen.findByRole("button", { name: "settings.memory.modeTitle" }),
    budget: await screen.findByRole("button", { name: "settings.memory.budgetTitle" }),
  };
}

describe("Réglages MEMORY", () => {
  afterEach(() => cleanup());
  beforeEach(() => resetSettingsTestEnvironment());

  it("enregistre le mode et le budget via les sélecteurs", async () => {
    const controls = await openMemorySettings();

    fireEvent.click(controls.mode);
    fireEvent.click(screen.getByRole("option", { name: "settings.memory.modes.automatic" }));
    await waitFor(() => expect(invokeCalls()).toContainEqual([
      "set_memory_mode",
      { mode: "automatic" },
    ]));

    fireEvent.click(controls.budget);
    fireEvent.click(screen.getByRole("option", { name: "2000 settings.memory.tokenUnit" }));
    await waitFor(() => expect(invokeCalls()).toContainEqual([
      "set_memory_context_budget",
      { tokens: 2000 },
    ]));
  });

  it("rétablit le mode précédent et permet de relancer après une erreur", async () => {
    const { mode } = await openMemorySettings();
    failInvokeCommand("set_memory_mode");

    fireEvent.click(mode);
    fireEvent.click(screen.getByRole("option", { name: "settings.memory.modes.manual" }));

    await waitFor(() => {
      expect(mode.textContent).toContain("settings.memory.modes.disabled");
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    restoreInvokeCommand("set_memory_mode");
    fireEvent.click(screen.getByText("settings.memory.retry"));

    await waitFor(() => {
      const refreshes = invokeCalls().filter(([command]) => command === "get_memory_overview");
      expect(refreshes.length).toBeGreaterThan(1);
      expect(screen.queryByRole("alert")).toBeNull();
    });
  });

  it("rétablit le budget précédent si son enregistrement échoue", async () => {
    const controls = await openMemorySettings();
    fireEvent.click(controls.mode);
    fireEvent.click(screen.getByRole("option", { name: "settings.memory.modes.automatic" }));
    await waitFor(() => expect(controls.budget).not.toBeDisabled());
    failInvokeCommand("set_memory_context_budget");

    fireEvent.click(controls.budget);
    fireEvent.click(screen.getByRole("option", { name: "2000 settings.memory.tokenUnit" }));

    await waitFor(() => {
      expect(controls.budget.textContent).toContain("3000 settings.memory.tokenUnit");
      expect(screen.getByRole("alert")).toBeTruthy();
    });
  });

  it("charge les sujets d’un autre projet uniquement à la demande", async () => {
    await openMemorySettings();

    fireEvent.click(await screen.findByText("settings.memory.loadTopics"));

    await waitFor(() => expect(invokeCalls()).toContainEqual([
      "get_memory_project_topics",
      { projectId: "bbbbbbbbbbbbbbbbbbbbbbbb" },
    ]));
    expect(await screen.findByText("Mémoire projet")).toBeTruthy();
  });
});
