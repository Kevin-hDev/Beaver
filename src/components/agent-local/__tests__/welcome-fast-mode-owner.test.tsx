import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import { AgentLocalTab } from "../agent-local-tab";

const owner = vi.hoisted(() => ({ create: vi.fn(), onSessionChange: vi.fn() }));

vi.hoisted(() => {
  Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
    configurable: true,
    value: () => null,
  });
});

vi.mock("react-i18next", async (importOriginal) => ({
  ...await importOriginal<typeof import("react-i18next")>(),
  useTranslation: () => ({
    t: (key: string) => ({
      "agentLocal.fastMode": "Rapide",
      "agentLocal.reasoningTitle": "Réflexion",
      "agentLocal.reasoningOff": "Désactivée",
      "agentLocal.reasoningHigh": "Forte",
      "agentLocal.newSession": "Nouvelle conversation",
    })[key] ?? key,
  }),
}));
vi.mock("@/hooks/use-agent-sessions", () => ({
  useAgentSessions: () => ({
    sessions: [], refresh: vi.fn(), create: owner.create, rename: vi.fn(),
    reorder: vi.fn(), reorderPinned: vi.fn(), remove: vi.fn(), archive: vi.fn(),
    togglePin: vi.fn(), updateModel: vi.fn(), updateReasoning: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-projects", () => ({
  useProjects: () => ({ projects: [], add: vi.fn(), remove: vi.fn() }),
}));
vi.mock("@/hooks/use-terminal", () => ({ useTerminal: () => ({}) }));
vi.mock("@/hooks/use-default-model", () => ({
  useDefaultModel: () => ({ model: "compatible", provider: "test-provider" }),
}));
vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups: new Map([["test-provider", [
    {
      id: "compatible", provider_id: "test-provider", provider_name: "Test",
      is_local: false, supports_tools: true, supports_thinking: true,
      supports_fast_mode: true, reasoning_modes: ["off", "high"],
    },
    {
      id: "incompatible", provider_id: "test-provider", provider_name: "Test",
      is_local: false, supports_tools: true, supports_thinking: true,
      supports_fast_mode: false, reasoning_modes: ["off", "high"],
    },
  ]]]) }),
}));
vi.mock("@/hooks/use-file-preview", () => ({ useFilePreview: () => ({}) }));
vi.mock("@/hooks/use-agent-local-shortcuts", () => ({ useAgentLocalShortcuts: vi.fn() }));
vi.mock("@/hooks/use-agent-local-preview-sync", () => ({ useAgentLocalPreviewSync: vi.fn() }));
vi.mock("@/hooks/use-agent-local-controlled-preview", () => ({
  useAgentLocalControlledPreview: () => ({ open: false, toggleOpen: vi.fn(), openPlan: vi.fn(), openOperation: vi.fn() }),
}));
vi.mock("@/hooks/use-agent-local-controlled-terminal", () => ({
  useAgentLocalControlledTerminal: () => ({ isOpen: false, tabs: [], addTab: vi.fn(), togglePanel: vi.fn() }),
}));
vi.mock("@/hooks/use-arrow-navigation", () => ({ useArrowNavigation: vi.fn() }));
vi.mock("@/hooks/use-unavailable-model-fallback", () => ({ useUnavailableModelFallback: vi.fn() }));
vi.mock("@/hooks/use-session-fast-mode", () => ({
  useSessionFastMode: () => ({ setFastMode: vi.fn(), isFastModePending: () => false }),
}));
vi.mock("@/hooks/use-directory-access-guard", () => ({
  useDirectoryAccessGuard: () => ({ prompt: null, request: vi.fn() }),
}));
vi.mock("@/hooks/use-file-tree", () => ({ useFileTree: () => ({}) }));
vi.mock("@/hooks/use-forecast-panel", () => ({ useForecastPanel: () => ({}) }));
vi.mock("@/hooks/use-agent-local-panel-nav", () => ({ useAgentLocalPanelNav: vi.fn() }));
vi.mock("@/hooks/use-agent-local-controlled-panels", () => ({
  useAgentLocalControlledPanels: () => ({ fileTreeNav: {}, forecastNav: { panelMode: "preview", setPanelMode: vi.fn() } }),
}));
vi.mock("@/hooks/use-git-branch", () => ({ useGitBranch: () => ({}) }));
vi.mock("@/hooks/use-git-uncommitted-files", () => ({ useGitUncommittedFiles: () => [] }));
vi.mock("@/hooks/use-session-summary", () => ({ useSessionSummary: () => ({}) }));
vi.mock("@/hooks/use-session-tabs", () => ({
  useSessionTabs: () => ({
    activeSessionId: null, activeTab: null, tabs: [], attentionTabIds: [],
    renameTab: vi.fn(), cloneMessage: vi.fn(), cancelCloneSummary: vi.fn(),
    createCloneGitBranch: vi.fn(), linkCloneGitBranch: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-agent-local-tab-git", () => ({
  useAgentLocalTabGit: () => ({ selectTab: vi.fn(), closeTab: vi.fn(), dialogs: null }),
}));
vi.mock("../use-agent-local-forecast-content", () => ({
  useAgentLocalForecastContent: () => ({
    forecastContent: null, fullscreenSwitching: false,
    handleOpenForecastDocs: vi.fn(), handlePreviewFullscreenChange: vi.fn(),
  }),
}));
vi.mock("../use-agent-local-conversation-list", () => ({ useAgentLocalConversationList: () => null }));
vi.mock("@/hooks/use-available-panel-mode", () => ({
  useAvailablePanelMode: () => ({ panelMode: "preview", browserStatus: "unavailable" }),
}));
vi.mock("@/components/ui/panel-slots", () => ({
  PanelSlot: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));
vi.mock("../chat-header", () => ({ ChatHeader: () => null }));
vi.mock("../model-selector", () => ({
  ModelSelector: ({ onSelect }: { onSelect: (model: string, provider: string) => void }) => (
    <>
      <button type="button" onClick={() => onSelect("incompatible", "test-provider")}>Modèle incompatible</button>
      <button type="button" onClick={() => onSelect("compatible", "test-provider")}>Modèle compatible</button>
    </>
  ),
}));
vi.mock("../welcome-view", async () => {
  const { ModelControls } = await import("../model-controls");
  return {
    WelcomeView: (props: {
      model: string;
      provider: string;
      reasoningMode?: string | null;
      fastModeEnabled: boolean;
      onModelChange: (model: string, provider: string) => void;
      onReasoningModeChange: (mode: ReasoningMode) => void;
      onFastModeChange: (enabled: boolean) => void;
    }) => (
      <ModelControls
        selectedModel={props.model}
        selectedProvider={props.provider}
        onSelect={props.onModelChange}
        reasoningMode={props.reasoningMode}
        onReasoningModeChange={props.onReasoningModeChange}
        fastModeEnabled={props.fastModeEnabled}
        fastModePending={false}
        onFastModeChange={props.onFastModeChange}
      />
    ),
  };
});

const navState = { ...DEFAULT_AGENT_LOCAL_NAV, sessionId: null };

function OwnerActionsHarness() {
  const state = useAgentLocalTab({ navState, listFocused: false, onSessionChange: owner.onSessionChange });
  return (
    <>
      <output aria-label="propriétaire Rapide">{String(state.welcomeFastModeEnabled)}</output>
      <button type="button" onClick={() => state.setWelcomeFastModeEnabled(true)}>Activer Rapide</button>
      <button type="button" onClick={() => void state.sessionActions.handleWelcomeSend("Bonjour").catch(() => undefined)}>Créer</button>
    </>
  );
}

describe("propriétaire du brouillon Rapide", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    owner.create.mockResolvedValue({ id: "created" });
  });

  it("conserve le brouillon pendant incompatible puis le restaure au retour compatible", () => {
    render(<AgentLocalTab navState={navState} />);
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));
    fireEvent.click(screen.getByRole("switch", { name: "Rapide" }));
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();

    fireEvent.click(screen.getByRole("button", { name: "Modèle incompatible" }));
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));
    expect(screen.queryByRole("switch", { name: "Rapide" })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Modèle compatible" }));
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();
  });

  it("ne remet pas le propriétaire à false quand la création échoue", async () => {
    owner.create.mockRejectedValue(new Error("échec interne"));
    render(<OwnerActionsHarness />);
    fireEvent.click(screen.getByRole("button", { name: "Activer Rapide" }));

    fireEvent.click(screen.getByRole("button", { name: "Créer" }));

    await waitFor(() => expect(owner.create).toHaveBeenCalled());
    expect(screen.getByRole("status", { name: "propriétaire Rapide" })).toHaveTextContent("true");
  });

  it("remet le propriétaire à false après une création réussie", async () => {
    render(<OwnerActionsHarness />);
    fireEvent.click(screen.getByRole("button", { name: "Activer Rapide" }));

    fireEvent.click(screen.getByRole("button", { name: "Créer" }));

    await waitFor(() => expect(screen.getByRole("status", { name: "propriétaire Rapide" })).toHaveTextContent("false"));
  });
});
