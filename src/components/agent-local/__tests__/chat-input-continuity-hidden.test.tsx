import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { ChatInputActionsRow } from "../chat-input-actions-row";
import { SlotProvider } from "@/features/extension-ui/slot-provider";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups: new Map() }),
}));
vi.mock("@/hooks/use-session-compression-profile", () => ({
  useSessionCompressionProfile: () => ({
    profiles: [],
    profilesStatus: "ready",
    effective: undefined,
    select: vi.fn(),
  }),
}));
vi.mock("../chat-plus-menu", () => ({ ChatPlusMenu: () => null }));
vi.mock("../context-progress", () => ({ ContextProgress: () => null }));
vi.mock("../permission-mode-selector", () => ({ PermissionModeSelector: () => null }));
vi.mock("../missing-directory-prompt", () => ({ MissingDirectoryPrompt: () => null }));
vi.mock("../plan-mode-badge", () => ({ PlanModeBadge: () => null }));
vi.mock("../retry-indicator", () => ({ RetryIndicator: () => null }));
vi.mock("../send-stop-button", () => ({ SendStopButton: () => null }));
vi.mock("../model-selector", () => ({ ModelSelector: () => <span>model</span> }));
vi.mock("../reasoning-selector", () => ({ ReasoningSelector: () => <span>reasoning</span> }));
vi.mock("react-i18next", async (importOriginal) => ({
  ...await importOriginal<typeof import("react-i18next")>(),
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ChatInputActionsRow", () => {
  it("ne montre aucun contrôle de continuité du raisonnement dans le composeur", () => {
    render(
      <SlotProvider>
        <ChatInputActionsRow
          sessionId="session-1"
          modelName="gemma4:e2b-it-q4_K_M"
          providerName="ollama"
          fastModeEnabled={false}
          fastModePending={false}
          contextUsed={0}
          contextMax={1}
          permissionMode="auto"
          planModeEnabled={false}
          buttonState="send"
          onPermissionModeChange={vi.fn()}
          onFileImport={vi.fn()}
          onModelChange={vi.fn()}
          onReasoningModeChange={vi.fn()}
          onFastModeChange={vi.fn()}
          onSend={vi.fn()}
          onStop={vi.fn()}
        />
      </SlotProvider>,
    );

    expect(invoke).not.toHaveBeenCalledWith("get_agent_session", { id: "session-1" });
    expect(screen.queryByRole("radiogroup", { name: "agentLocal.continuityTitle" })).toBeNull();
    expect(screen.queryByText("agentLocal.continuityOptional")).toBeNull();
  });
});
