import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import type { AgentSessionMeta } from "@/types/agent";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { useSessionTabs } from "@/hooks/use-session-tabs";
import { SlotProvider } from "@/features/extension-ui/slot-provider";
import { AgentLocalTab } from "../agent-local-tab";

vi.mock("@/hooks/use-agent-local-tab", () => ({ useAgentLocalTab: vi.fn() }));
vi.mock("@/hooks/use-session-tabs", () => ({ useSessionTabs: vi.fn() }));
vi.mock("@/hooks/use-file-tree", () => ({ useFileTree: vi.fn(() => ({})) }));
vi.mock("@/hooks/use-forecast-panel", () => ({ useForecastPanel: vi.fn(() => ({})) }));
vi.mock("@/hooks/use-agent-local-panel-nav", () => ({ useAgentLocalPanelNav: vi.fn() }));
vi.mock("@/hooks/use-agent-local-controlled-panels", () => ({
  useAgentLocalControlledPanels: vi.fn(() => ({
    fileTreeNav: {},
    forecastNav: { panelMode: "preview", setPanelMode: vi.fn() },
  })),
}));
vi.mock("@/hooks/use-git-branch", () => ({ useGitBranch: vi.fn(() => ({})) }));
vi.mock("@/hooks/use-git-uncommitted-files", () => ({ useGitUncommittedFiles: vi.fn(() => []) }));
vi.mock("@/hooks/use-session-summary", () => ({ useSessionSummary: vi.fn(() => ({})) }));
vi.mock("@/hooks/use-agent-local-tab-git", () => ({
  useAgentLocalTabGit: vi.fn(() => ({ selectTab: vi.fn(), closeTab: vi.fn(), dialogs: null })),
}));
vi.mock("../use-agent-local-forecast-content", () => ({
  useAgentLocalForecastContent: vi.fn(() => ({
    forecastContent: null,
    fullscreenSwitching: false,
    handleOpenForecastDocs: vi.fn(),
    handlePreviewFullscreenChange: vi.fn(),
  })),
}));
vi.mock("../use-agent-local-conversation-list", () => ({ useAgentLocalConversationList: vi.fn(() => null) }));
vi.mock("@/hooks/use-available-panel-mode", () => ({
  useAvailablePanelMode: vi.fn(() => ({ panelMode: "preview", browserStatus: "unavailable" })),
}));
vi.mock("@/components/layout/panel-slots", () => ({
  PanelSlot: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));
vi.mock("../chat-header", () => ({ ChatHeader: () => null }));
vi.mock("../welcome-view", () => ({
  WelcomeView: (props: {
    fastModeEnabled?: boolean;
    onFastModeChange?: (enabled: boolean) => void;
  }) => (
    <input
      type="checkbox"
      role="switch"
      aria-label="Rapide"
      checked={props.fastModeEnabled ?? false}
      onChange={(event) => props.onFastModeChange?.(event.target.checked)}
    />
  ),
}));
vi.mock("react-i18next", () => ({
  initReactI18next: { type: "3rdParty", init: vi.fn() },
  useTranslation: () => ({
    t: (key: string) => ({
      "agentLocal.fastMode": "Rapide",
      "agentLocal.reasoningTitle": "Réflexion",
      "agentLocal.reasoningOff": "Désactivée",
      "agentLocal.reasoningHigh": "Forte",
    })[key] ?? key,
  }),
}));
vi.mock("@/hooks/use-agent-panel-layout", () => ({
  useAgentPanelLayout: () => ({
    containerRef: { current: null },
    layout: { chatMinWidth: 420, previewWidth: 0, fileTreeWidth: 0 },
  }),
}));
vi.mock("@/components/agent-side-panel/agent-side-panel", () => ({ AgentSidePanel: () => null }));
vi.mock("@/components/file-preview/file-preview-panel", () => ({ FilePreviewPanel: () => null }));
vi.mock("@/components/file-tree/file-tree-panel", () => ({ FileTreePanel: () => null }));
vi.mock("@/components/internal-browser/browser-panel", () => ({ BrowserPanel: () => null }));
vi.mock("../chat-transcript", () => ({ ChatTranscript: () => null }));
vi.mock("../chat-input-footer", () => ({ ChatInputFooter: () => null }));
vi.mock("../chat-terminal-dock", () => ({ ChatTerminalDock: () => null }));
vi.mock("../todo-progress-panel", () => ({ TodoProgressPanel: () => null }));
vi.mock("../subagent-accordion", () => ({ SubagentAccordion: () => null }));
vi.mock("../permission-dialog", () => ({ PermissionDialog: () => null }));
vi.mock("../error-bubble", () => ({ ErrorBubble: () => null }));
vi.mock("../scroll-bottom-button", () => ({ ScrollBottomButton: () => null }));
vi.mock("../chat-overlays", () => ({ ChatOverlays: () => null }));
vi.mock("../file-drop-zone", () => ({
  FileDropZone: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));
vi.mock("../chat-input-editor", () => ({ ChatInputEditor: () => null }));
vi.mock("../slash-autocomplete", () => ({ SlashAutocomplete: () => null }));
vi.mock("../file-thumbnail", () => ({ FileThumbnail: () => null }));
vi.mock("../interactive-choice-panel", () => ({ InteractiveChoicePanel: () => null }));
vi.mock("../chat-plus-menu", () => ({ ChatPlusMenu: () => null }));
vi.mock("../context-progress", () => ({ ContextProgress: () => null }));
vi.mock("../permission-mode-selector", () => ({ PermissionModeSelector: () => null }));
vi.mock("../missing-directory-prompt", () => ({ MissingDirectoryPrompt: () => null }));
vi.mock("../plan-mode-badge", () => ({ PlanModeBadge: () => null }));
vi.mock("../retry-indicator", () => ({ RetryIndicator: () => null }));
vi.mock("../send-stop-button", () => ({ SendStopButton: () => null }));
vi.mock("../model-selector", () => ({ ModelSelector: () => null }));
vi.mock("@/hooks/use-agent-chat", () => ({
  useAgentChat: () => ({
    messages: [], sessionTokenCount: 0, contextLimitTokens: 0, planModeEnabled: false,
    completedSegments: [], currentTools: [], currentContent: "", currentContentPhase: undefined,
    currentThinking: "", isStreaming: false, planPreview: null, sessionLoading: false,
    contextUsageVisible: false, forbiddenAllowedPaths: [], dismissForbiddenDirectory: vi.fn(),
    error: null, interactiveChoice: null, clearInteractiveChoice: vi.fn(),
    missingDirectory: null, missingDirectoryResolving: false, resolveMissingDirectory: vi.fn(),
    setPlanModeEnabled: vi.fn(), stop: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-context-progress", () => ({ useContextProgress: () => ({ max: 8_000 }) }));
vi.mock("@/hooks/use-context-usage", () => ({ useContextUsage: () => ({ used: 0 }) }));
vi.mock("@/hooks/use-file-drop", () => ({
  useFileDrop: () => ({ dragging: false, setDragging: vi.fn(), addByPaths: vi.fn(), files: [], removeFile: vi.fn(), clearFiles: vi.fn() }),
}));
vi.mock("@/hooks/use-permission-mode", () => ({
  usePermissionMode: () => ({ mode: "auto", refresh: vi.fn(), availableModes: [], change: vi.fn() }),
}));
vi.mock("@/hooks/use-permission-requests", () => ({ usePermissionRequests: () => ({ enqueue: vi.fn(), current: null, respond: vi.fn() }) }));
vi.mock("@/hooks/use-session-project", () => ({ useSessionProject: () => ({ selectedProject: undefined, selectedProjectId: undefined }) }));
vi.mock("@/hooks/use-chat-scroll", () => ({ useChatScroll: () => ({ containerRef: { current: null }, isAtBottom: true, scrollToBottom: vi.fn() }) }));
vi.mock("@/hooks/use-model-switch", () => ({ useModelSwitch: () => ({ pendingSwitch: null, setPendingSwitch: vi.fn(), handleModelSelect: vi.fn(), rememberedRef: { current: null } }) }));
vi.mock("@/hooks/use-worktree-session-switch", () => ({ useWorktreeSessionSwitch: () => ({ pending: null, request: vi.fn(), cancel: vi.fn(), createSession: vi.fn() }) }));
vi.mock("@/hooks/use-session-files", () => ({ useSessionFileGroups: vi.fn() }));
vi.mock("@/hooks/use-subagents", () => ({ useSubagents: () => ({ active: [], completed: [], cancelSubagent: vi.fn() }) }));
vi.mock("@/hooks/use-chat-actions", () => ({ useChatActions: () => ({ handleSend: vi.fn(), handleFileImport: vi.fn() }) }));
vi.mock("@/hooks/use-chat-clone", () => ({ useChatClone: () => ({ requestClone: vi.fn(), summaryRun: null, pendingClone: null, cloneBusy: false, cancelClone: vi.fn(), abortClone: vi.fn(), submitClone: vi.fn() }) }));
vi.mock("@/hooks/use-clone-git-branch-action", () => ({ useCloneGitBranchAction: () => undefined }));
vi.mock("@/hooks/use-selected-model-capabilities", () => ({ useSelectedModelCapabilities: () => ({ supports_tools: true }) }));
vi.mock("@/hooks/use-chat-view-runtime", () => ({ useChatViewRuntime: () => ({ showError: false, retryIndicator: null }) }));
vi.mock("@/hooks/use-preflight-directory-access-prompt", () => ({ usePreflightDirectoryAccessPrompt: () => undefined }));
vi.mock("@/hooks/use-composer-handoff", () => ({ useComposerHandoff: vi.fn() }));
vi.mock("@/lib/composer-handoff", () => ({ hasComposerPosition: () => false }));
vi.mock("@/hooks/use-slash-commands", () => ({ useSlashCommands: () => ({ showDropdown: false, skills: [], activeIndex: 0, handleInput: vi.fn(), close: vi.fn() }) }));
vi.mock("@/hooks/use-active-skills", () => ({ useActiveSkills: () => ({ activeSkills: [], getSkillsPayload: () => [] }) }));
vi.mock("../use-interactive-choice-feedback", () => ({ useInteractiveChoiceFeedback: () => ({ error: null, resolve: vi.fn(), fail: vi.fn() }) }));
vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups: new Map([["codex-oauth", [{
    id: "gpt-5.6-sol", provider_id: "codex-oauth", provider_name: "OpenAI",
    is_local: false, supports_tools: true, supports_thinking: true,
    supports_fast_mode: true, reasoning_modes: ["off", "high"],
  }]]]) }),
}));

const rootSession: AgentSessionMeta = {
  id: "root",
  name: "Racine",
  model: "gpt-5.6-sol",
  provider: "codex-oauth",
  fast_mode_enabled: false,
  message_count: 1,
  created_at: "2026-08-23T00:00:00Z",
};
const cloneSession: AgentSessionMeta = {
  ...rootSession,
  id: "clone",
  name: "Clone",
  fast_mode_enabled: true,
  clone_parent_session_id: "root",
};

let displayedSessionId: string | null = "clone";
let displayedSessions: AgentSessionMeta[] = [rootSession, cloneSession];
let welcomeMode = false;
let pendingSessionId: string | null = null;
const setFastMode = vi.fn();
const setWelcomeFastModeEnabled = vi.fn((enabled: boolean) => { welcomeMode = enabled; });

function localTabState() {
  const activeSession = displayedSessionId === null ? null : rootSession;
  return {
    sessions: displayedSessions,
    refresh: vi.fn(),
    archive: vi.fn(),
    updateModel: vi.fn(),
    projectsHook: { projects: [], add: vi.fn() },
    terminal: { isOpen: false, tabs: [], addTab: vi.fn(), togglePanel: vi.fn() },
    activeSession,
    activeSessionId: activeSession?.id ?? null,
    model: "gpt-5.6-sol",
    provider: "codex-oauth",
    currentDefault: { model: "gpt-5.6-sol", provider: "codex-oauth" },
    activeProject: null,
    filePreview: {
      open: false, width: 0, extraWidth: 0, fullscreen: false, resizing: false,
      tabs: [], activeTab: null, listMode: "all", toggleOpen: vi.fn(), openPlan: vi.fn(),
      openOperation: vi.fn(), openPath: vi.fn(), openFullPath: vi.fn(), startResize: vi.fn(),
      setActiveTab: vi.fn(), setListMode: vi.fn(), closeTab: vi.fn(),
    },
    fileOperations: { all: [], latest: [] },
    setFileOperations: vi.fn(),
    reasoningMode: "high",
    setReasoningMode: vi.fn(),
    welcomeModel: null,
    setWelcomeModel: vi.fn(),
    welcomeFastModeEnabled: welcomeMode,
    setWelcomeFastModeEnabled,
    setFastMode,
    isFastModePending: (id: string) => id === pendingSessionId,
    sessionActions: {
      pendingMessage: null,
      setPendingMessage: vi.fn(),
      pendingWorkingDir: undefined,
      setPendingWorkingDir: vi.fn(),
      pendingSkills: undefined,
      setPendingSkills: vi.fn(),
      pendingFiles: undefined,
      setPendingFiles: vi.fn(),
      handleCreateWithModel: vi.fn(),
      handleWelcomeSend: vi.fn(),
      handleAutoRename: vi.fn(),
      handleCreateInProjectWithModel: vi.fn(),
    },
    handleSelectById: vi.fn(),
  };
}

function renderTab() {
  return render(tab());
}

function tab(sessionId: string | null = "root") {
  return (
    <SlotProvider>
      <AgentLocalTab navState={{ ...DEFAULT_AGENT_LOCAL_NAV, sessionId }} />
    </SlotProvider>
  );
}

describe("AgentLocalTab Rapide", () => {
  beforeEach(() => {
    displayedSessionId = "clone";
    displayedSessions = [rootSession, cloneSession];
    welcomeMode = false;
    pendingSessionId = null;
    vi.clearAllMocks();
    vi.mocked(useAgentLocalTab).mockImplementation(() => localTabState() as never);
    vi.mocked(useSessionTabs).mockImplementation(() => ({
      activeSessionId: displayedSessionId,
      activeTab: null,
      tabs: [],
      attentionTabIds: [],
      renameTab: vi.fn(),
      cloneMessage: vi.fn(),
      cancelCloneSummary: vi.fn(),
      createCloneGitBranch: vi.fn(),
      linkCloneGitBranch: vi.fn(),
    }) as never);
  });

  afterEach(cleanup);

  it("lit et modifie le clone affiché plutôt que la session racine", () => {
    const view = renderTab();
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));
    const cloneSwitch = screen.getByRole("switch", { name: "Rapide" });

    expect(cloneSwitch).toBeChecked();
    expect(cloneSwitch).not.toBeDisabled();
    fireEvent.click(cloneSwitch);
    expect(setFastMode).toHaveBeenCalledWith("clone", false);

    displayedSessionId = "root";
    view.rerender(tab());
    const rootSwitch = screen.getByRole("switch", { name: "Rapide" });
    expect(rootSwitch).not.toBeChecked();
    expect(rootSwitch).not.toBeDisabled();

    fireEvent.click(rootSwitch);
    expect(setFastMode).toHaveBeenCalledWith("root", true);
  });

  it("désactive uniquement la session dont la sauvegarde est en attente", () => {
    pendingSessionId = "clone";
    renderTab();
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));

    expect(screen.getByRole("switch", { name: "Rapide" })).toBeDisabled();
  });

  it("reflète les métadonnées confirmées après rechargement", () => {
    const view = renderTab();
    fireEvent.click(screen.getByRole("button", { name: /Réflexion/ }));
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();

    displayedSessions = [rootSession, { ...cloneSession, fast_mode_enabled: false }];
    view.rerender(tab());

    expect(screen.getByRole("switch", { name: "Rapide" })).not.toBeChecked();
  });

  it("utilise un brouillon d'accueil false indépendant des sessions", () => {
    displayedSessionId = null;
    const view = renderTab();
    const welcomeSwitch = screen.getByRole("switch", { name: "Rapide" });

    expect(welcomeSwitch).not.toBeChecked();
    fireEvent.click(welcomeSwitch);
    expect(setWelcomeFastModeEnabled).toHaveBeenCalledWith(true);

    view.rerender(tab(null));
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();
  });
});
