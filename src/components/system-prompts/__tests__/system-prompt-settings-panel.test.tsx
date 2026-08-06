import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
  selection: "default",
  nativePromptAvailable: false,
};

describe("SystemPromptSettingsPanel", () => {
  let clipboardWrite: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    localStorage.clear();
    vi.mocked(invoke).mockReset().mockResolvedValue(beaverView);
    clipboardWrite = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: clipboardWrite },
    });
  });

  it("explique comment récupérer des réglages de prompts illisibles", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("system-prompt-store-unavailable");
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="agentic"
        initialTier="detailed"
      />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "errors.localStore.systemPrompts",
    );
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
    expect(screen.getByRole("tab", { name: "settings.systemPrompt.tiers.compact < 25B" })).toBeVisible();

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
      selection: "disabled",
      nativePromptAvailable: false,
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
    expect(screen.getByRole("button", { name: "settings.systemPrompt.useBeaver" })).toBeVisible();

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

  it("garde les deux sélecteurs groupés sur une seule ligne", async () => {
    const { container } = render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "gemma4:e2b" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="compact"
      />,
    );
    await screen.findByText("Beaver instructions");

    const row = container.querySelector(".spp-selectors");
    expect(row).toContainElement(
      screen.getByRole("tab", { name: "settings.systemPrompt.modes.agentic" }),
    );
    expect(row).toContainElement(
      screen.getByRole("tab", { name: "settings.systemPrompt.tiers.compact < 25B" }),
    );
  });

  it("utilise Beaver seulement pour la combinaison sélectionnée", async () => {
    const customView: SystemPromptView = {
      content: "Custom instructions",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: false,
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

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useBeaver" }));

    expect(screen.getByRole("dialog")).toHaveTextContent(
      "settings.systemPrompt.loss.body",
    );
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.loss.continue" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restore_system_prompt_setting", {
        target: { scope: "global" },
        mode: "chatbot",
        tier: "detailed",
      });
      expect(screen.queryByRole("button", { name: "settings.systemPrompt.useBeaver" })).toBeNull();
    });
  });

  it("permet de choisir Beaver directement puis de restaurer le prompt Ollama", async () => {
    const nativeView: SystemPromptView = {
      content: "Native instructions",
      source: "ollama",
      selection: "default",
      nativePromptAvailable: true,
    };
    const forcedBeaverView: SystemPromptView = {
      ...beaverView,
      selection: "beaver",
      nativePromptAvailable: true,
    };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "restore_system_prompt_setting") return Promise.resolve(forcedBeaverView);
      if (command === "restore_default_system_prompt_setting") return Promise.resolve(nativeView);
      return Promise.resolve(nativeView);
    });
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "phi4:latest" }}
        warningKind="ollama"
        initialMode="chatbot"
        initialTier="compact"
      />,
    );
    await screen.findByText("Native instructions");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useBeaver" }));
    await screen.findByText("Beaver instructions");
    expect(screen.getByRole("button", { name: "settings.systemPrompt.useOllama" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useOllama" }));
    await screen.findByText("Native instructions");
    expect(invoke).toHaveBeenCalledWith("restore_default_system_prompt_setting", {
      target: { scope: "ollama", model: "phi4:latest" },
      mode: "chatbot",
      tier: "compact",
    });
  });

  it("avertit et permet de copier avant de remplacer un prompt personnalisé par Ollama", async () => {
    const customView: SystemPromptView = {
      content: "Custom instructions",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: true,
    };
    const nativeView: SystemPromptView = {
      content: "Native instructions",
      source: "ollama",
      selection: "default",
      nativePromptAvailable: true,
    };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "restore_default_system_prompt_setting") return Promise.resolve(nativeView);
      return Promise.resolve(customView);
    });
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "phi4:latest" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="detailed"
      />,
    );
    await screen.findByText("Custom instructions");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useOllama" }));

    expect(screen.getByRole("dialog")).toHaveTextContent(
      "settings.systemPrompt.loss.body",
    );
    expect(invoke).not.toHaveBeenCalledWith(
      "restore_default_system_prompt_setting",
      expect.anything(),
    );

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.loss.copy" }));
    await waitFor(() => {
      expect(clipboardWrite).toHaveBeenCalledWith("Custom instructions");
      expect(screen.getByRole("button", { name: "settings.systemPrompt.loss.copied" })).toBeVisible();
    });
    expect(screen.getByRole("dialog")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.loss.continue" }));
    await screen.findByText("Native instructions");
  });

  it("réarme le bouton de copie pour qu'un second clic reste lisible", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const customView: SystemPromptView = {
      content: "Instructions to copy twice",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: true,
    };
    vi.mocked(invoke).mockResolvedValue(customView);
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "phi4:latest" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="compact"
      />,
    );
    await screen.findByText("Instructions to copy twice");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useOllama" }));
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.loss.copy" }));
    await screen.findByRole("button", { name: "settings.systemPrompt.loss.copied" });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000);
    });

    expect(screen.getByRole("button", { name: "settings.systemPrompt.loss.copy" })).toBeVisible();
    vi.useRealTimers();
  });

  it("garde le dialogue ouvert si la copie échoue", async () => {
    const customView: SystemPromptView = {
      content: "Instructions to preserve",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: true,
    };
    clipboardWrite.mockRejectedValueOnce(new Error("clipboard unavailable"));
    vi.mocked(invoke).mockResolvedValue(customView);
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "phi4:latest" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="compact"
      />,
    );
    await screen.findByText("Instructions to preserve");

    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useOllama" }));
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.loss.copy" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "settings.systemPrompt.loss.copyError",
    );
    expect(screen.getByRole("dialog")).toBeVisible();
    expect(invoke).not.toHaveBeenCalledWith(
      "restore_default_system_prompt_setting",
      expect.anything(),
    );
  });

  it("annule sans perdre le prompt personnalisé", async () => {
    const customView: SystemPromptView = {
      content: "Keep these instructions",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: false,
    };
    vi.mocked(invoke).mockResolvedValue(customView);
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "global" }}
        warningKind="global"
        initialMode="chatbot"
        initialTier="compact"
      />,
    );
    await screen.findByText("Keep these instructions");

    expect(screen.queryByRole("button", { name: "settings.systemPrompt.restore" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "settings.systemPrompt.useBeaver" }));
    const cancelButton = screen.getByRole("button", { name: "settings.systemPrompt.cancel" });
    expect(cancelButton).toHaveFocus();
    fireEvent.click(cancelButton);

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText("Keep these instructions")).toBeVisible();
    expect(invoke).not.toHaveBeenCalledWith(
      "restore_system_prompt_setting",
      expect.anything(),
    );
  });

  it("ne propose jamais Ollama quand aucun prompt natif vérifié n’existe", async () => {
    const customView: SystemPromptView = {
      content: "Test system prompt",
      source: "custom",
      selection: "custom",
      nativePromptAvailable: false,
    };
    vi.mocked(invoke).mockResolvedValue(customView);
    render(
      <SystemPromptSettingsPanel
        target={{ scope: "ollama", model: "gemma4:e2b" }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier="compact"
      />,
    );
    await screen.findByText("Test system prompt");

    expect(screen.getByRole("button", { name: "settings.systemPrompt.useBeaver" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "settings.systemPrompt.useOllama" })).toBeNull();
    expect(screen.queryByText("settings.systemPrompt.restoreDefault")).toBeNull();
  });
});
