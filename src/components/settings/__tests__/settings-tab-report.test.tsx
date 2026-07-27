/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { SettingsSubTab } from "@/types/navigation";
import {
  CHILD_COMMANDS,
  invokeCalls,
  invokedCommands,
  resetSettingsTestEnvironment,
  setXaiOAuthState,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

describe("SettingsTab slots", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    resetSettingsTestEnvironment();
  });

  it("ne monte pas les sous-onglets sur les pages principales", async () => {
    render(<StrictMode><SettingsHarness /></StrictMode>);

    await screen.findAllByText("settings.tabs.general");
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(invokedCommands().some((cmd) => CHILD_COMMANDS.has(cmd))).toBe(false);
  });

  it.each([
    ["ollama", "settings.tabs.ollama", "llama3.2:latest"],
    ["connectors", "settings.tabs.connectors", "Canva"],
    ["channels", "settings.tabs.channels", "test-telegram"],
    ["providers", "settings.tabs.providers", "Groq"],
    ["forecast", "forecast.title", "Chronos Bolt Small"],
  ] as Array<[SettingsSubTab, string, string]>)("ouvre %s sans crash ni boucle", async (_subTab, label, expectedContent) => {
    render(<StrictMode><SettingsHarness /></StrictMode>);
    const [item] = await screen.findAllByText(label);
    fireEvent.click(item);

    await waitFor(() => {
      const active = screen.getAllByText(label)
        .some((element) => element.closest('[role="button"]')?.getAttribute("data-nav-active") === "true");
      expect(active).toBe(true);
    });

    await waitFor(() => expect(screen.getAllByText(expectedContent).length).toBeGreaterThan(0));
  });

  it("bascule de Clés API vers OAuth avec une sidebar contextuelle", async () => {
    render(<StrictMode><SettingsHarness /></StrictMode>);
    fireEvent.click((await screen.findAllByText("settings.tabs.providers"))[0]);

    expect(await screen.findByText("providers.tabs.apiKeys")).toBeTruthy();
    fireEvent.click(screen.getByText("providers.tabs.oauth"));

    await waitFor(() => expect(screen.getAllByText("OpenAI").length).toBeGreaterThan(0));
    expect(screen.getByTestId("settings-detail").querySelector(".prv-subtabs")).toBeTruthy();
    expect(screen.getByTestId("settings-detail").querySelector(".prv-oauth-inner")).toBeTruthy();
    expect(screen.queryByText("Moonshot AI")).toBeNull();

    fireEvent.click(screen.getByText("providers.oauth.openCatalog"));
    expect(await screen.findByText("Moonshot AI")).toBeTruthy();
    fireEvent.click(screen.getByText("Moonshot AI"));
    await waitFor(() => {
      const call = invokeCalls().find(([command]) => command === "start_oauth_provider_login");
      const args = call?.[1] as Record<string, unknown> | undefined;
      expect(args?.providerId).toBe("moonshot");
      expect(typeof args?.diagnosticId).toBe("string");
    });
    expect(screen.getByText("connectors.oauth.title")).toBeTruthy();
    expect(screen.queryByText("providers.oauth.install")).toBeNull();
    expect(within(screen.getByTestId("settings-list")).queryByText("Moonshot AI")).toBeNull();
  });

  it("ne montre plus la connexion OpenAI dans Avancé", async () => {
    render(<StrictMode><SettingsHarness /></StrictMode>);
    fireEvent.click((await screen.findAllByText("settings.tabs.advanced"))[0]);

    await waitFor(() => expect(screen.queryByText("codex.title")).toBeNull());
  });

  it("ouvre la page Mascotte et son aperçu", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.mascot"))[0]);

    expect(await screen.findByLabelText("settings.mascot.previewTitle")).toBeTruthy();
    expect(screen.getAllByText("settings.mascot.beaverName").length).toBeGreaterThan(0);
  });

  it("sélectionne Circuit depuis une carte sans liste déroulante", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.mascot"))[0]);

    const circuitCard = (await screen.findByText("settings.mascot.circuitName")).closest("button");
    expect(circuitCard).toHaveAttribute("aria-pressed", "false");
    expect(screen.queryByRole("combobox")).toBeNull();
    fireEvent.click(circuitCard!);

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "patch_mascot_settings",
        { patch: { mascot_id: "circuit" } },
      ]);
      expect(circuitCard).toHaveAttribute("aria-pressed", "true");
      expect(document.querySelector(".msp-bubble [data-mascot-id='circuit']")).toBeTruthy();
    });
  });

  it("garde les mémoires consultables quand leur utilisation est désactivée", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.memory"))[0]);

    expect(await screen.findByText("settings.memory.disabledHint")).toBeTruthy();
    fireEvent.click(await screen.findByText("Interface compacte"));

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "read_file_preview",
        { path: "/memory/global/topics/interface.md", baseDir: null },
      ]);
    });
    await waitFor(() => {
      expect(document.querySelector(".mems-preview")?.textContent)
        .toContain("Interface compacte");
    });
  });

  it("archive une mémoire seulement après confirmation", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.memory"))[0]);
    fireEvent.click(await screen.findByText("Interface compacte"));
    await waitFor(() => expect(document.querySelector(".mems-preview")).toBeTruthy());

    fireEvent.click(screen.getByText("settings.memory.archive"));
    expect(invokedCommands()).not.toContain("archive_memory_topic");
    fireEvent.click(screen.getByText("settings.memory.confirmArchive"));

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "archive_memory_topic",
        { path: "/memory/global/topics/interface.md", sessionId: null },
      ]);
    });
    await waitFor(() => expect(screen.queryByText("Interface compacte")).toBeNull());
  });

  it("lance Grok OAuth sans installation de client", async () => {
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.providers"))[0]);
    fireEvent.click(await screen.findByText("providers.tabs.oauth"));
    fireEvent.click(await screen.findByText("providers.oauth.openCatalog"));
    fireEvent.click(await screen.findByText("xAI"));
    await waitFor(() => {
      const call = invokeCalls().find(([command]) => command === "start_oauth_provider_login");
      const args = call?.[1] as Record<string, unknown> | undefined;
      expect(args?.providerId).toBe("xai");
      expect(typeof args?.diagnosticId).toBe("string");
    }, { timeout: 3500 });
  });

  it("retire immédiatement un provider OAuth après sa déconnexion", async () => {
    setXaiOAuthState({ ready: true, connected: true });
    render(<SettingsHarness />);
    fireEvent.click((await screen.findAllByText("settings.tabs.providers"))[0]);
    fireEvent.click(await screen.findByText("providers.tabs.oauth"));
    fireEvent.click((await screen.findAllByText("xAI"))[0]);
    fireEvent.click(await screen.findByText("providers.oauth.disconnect"));

    await waitFor(() => {
      expect(within(screen.getByTestId("settings-list")).queryByText("xAI")).toBeNull();
    });
  });

  it("ouvre Tools avec tools verrouillés, optionnels et état grisé", async () => {
    render(<StrictMode><SettingsHarness /></StrictMode>);
    const [item] = await screen.findAllByText("settings.tabs.tools");
    fireEvent.click(item);

    const webRow = (await screen.findByText("settings.tools.groups.web.title")).closest(".settings-row");
    const skillsRow = screen.getByText("settings.tools.groups.skills.title").closest(".settings-row");
    const forecastRow = screen.getByText("settings.tools.groups.forecast.title").closest(".settings-row");

    expect(webRow?.querySelector("input")).toBeNull();
    expect(skillsRow ? within(skillsRow as HTMLElement).getByRole("switch") : null).toBeTruthy();
    expect(forecastRow).toHaveClass("is-off");

    const skillsToggle = within(skillsRow as HTMLElement).getByRole("switch");
    fireEvent.click(skillsToggle);

    await waitFor(() => {
      expect(invokeCalls()).toContainEqual([
        "set_agent_tool_group_enabled",
        { groupId: "skills", enabled: false },
      ]);
    });
  });
});
