import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SystemPromptSettingsPanel } from "../system-prompt-settings-panel";
import type { SystemPromptView } from "@/types/system-prompts";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const beaverView: SystemPromptView = {
  content: "Beaver instructions",
  source: "beaver",
  customized: false,
  disabled: false,
};

describe("SystemPromptSettingsPanel", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockReset().mockResolvedValue(beaverView);
  });

  it.each(["dark", "light"])("conserve sa carte avec le thème %s", async (theme) => {
    const { container } = render(
      <div data-theme={theme}>
        <SystemPromptSettingsPanel
          target={{ scope: "global" }}
          warningKind="global"
          initialMode="agentic"
          initialTier="detailed"
        />
      </div>,
    );

    await screen.findByText("Beaver instructions");
    expect(container.querySelector(`[data-theme="${theme}"] .settings-card.spp-card`)).toBeTruthy();
  });

  it("permet de choisir indépendamment le mode et le format", async () => {
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="agentic"
        initialTier="detailed"
      />,
    );
    await screen.findByText("Beaver instructions");

    fireEvent.click(screen.getByRole("tab", { name: "settings.systemPrompt.modes.chatbot" }));
    fireEvent.click(screen.getByRole("tab", { name: /settings.systemPrompt.tiers.compact/ }));

    await waitFor(() => {
      expect(invoke).toHaveBeenLastCalledWith("get_system_prompt_setting", {
        target: { scope: "global" },
        mode: "chatbot",
        tier: "compact",
      });
    });
  });

  it("abandonne le brouillon avant d’afficher une autre combinaison", async () => {
    vi.mocked(invoke).mockImplementation((_command, args) => {
      const selection = args as { mode?: string } | undefined;
      return Promise.resolve({
        ...beaverView,
        content: selection?.mode === "chatbot" ? "Chatbot instructions" : "Agent instructions",
      });
    });
    localStorage.setItem("system-prompt-warning-global-v1", "1");
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="agentic"
        initialTier="detailed"
      />,
    );
    await screen.findByText("Agent instructions");
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.edit" }));
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Draft" } });

    fireEvent.click(screen.getByRole("tab", { name: "settings.systemPrompt.modes.chatbot" }));

    await screen.findByText("Chatbot instructions");
    expect(screen.queryByRole("textbox")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.edit" }));
    expect(screen.getByRole("textbox")).toHaveValue("Chatbot instructions");
  });

  it("garde le champ vide après suppression et réouverture", async () => {
    const disabledView: SystemPromptView = {
      content: "",
      source: "custom",
      customized: true,
      disabled: true,
    };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "save_system_prompt_setting") return Promise.resolve(disabledView);
      return Promise.resolve(beaverView);
    });
    localStorage.setItem("system-prompt-warning-global-v1", "1");
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="agentic"
        initialTier="compact"
      />,
    );
    await screen.findByText("Beaver instructions");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.edit" }));
    fireEvent.change(screen.getByRole("textbox", { name: "settings.systemPrompt.editorLabel" }), {
      target: { value: "" },
    });
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.save" }));

    await screen.findByText("settings.systemPrompt.empty");
    expect(screen.getByRole("button", { name: "settings.systemPrompt.restore" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.edit" }));
    expect(screen.getByRole("textbox", { name: "settings.systemPrompt.editorLabel" })).toHaveValue("");
  });

  it("affiche un avertissement propre à chaque portée", async () => {
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "phi4:latest" }}
        warningKind="ollama"
        initialMode="chatbot"
        initialTier="compact"
      />,
    );
    await screen.findByText("Beaver instructions");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.edit" }));

    expect(screen.getByRole("dialog")).toHaveTextContent(
      "settings.systemPrompt.warning.ollama.body",
    );
    expect(screen.queryByText("settings.systemPrompt.warning.global.body")).toBeNull();
    expect(listen).toHaveBeenCalledWith("modelfile-updated", expect.any(Function));
  });

  it("peut placer le mode à côté de l’en-tête du modèle Ollama", async () => {
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "gemma4:e2b" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="compact"
        selectorHeader={<span>gemma4:e2b</span>}
        selectorActions={<button type="button">Model actions</button>}
      />,
    );
    await screen.findByText("Beaver instructions");

    const row = screen.getByText("gemma4:e2b").closest(".spp-mode-row");
    expect(row).toContainElement(
      screen.getByRole("tab", { name: "settings.systemPrompt.modes.agentic" }),
    );
    expect(row).toContainElement(screen.getByRole("button", { name: "Model actions" }));
  });

  it("restaure seulement la combinaison sélectionnée", async () => {
    const customView: SystemPromptView = {
      content: "Custom instructions",
      source: "custom",
      customized: true,
      disabled: false,
    };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "restore_system_prompt_setting") return Promise.resolve(beaverView);
      return Promise.resolve(customView);
    });
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="chatbot"
        initialTier="detailed"
      />,
    );
    await screen.findByText("Custom instructions");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.restore" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restore_system_prompt_setting", {
        target: { scope: "global" },
        mode: "chatbot",
        tier: "detailed",
      });
      expect(screen.queryByRole("button", { name: "settings.systemPrompt.restore" })).toBeNull();
    });
  });
});
