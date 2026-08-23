import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import type { AgentSessionMeta } from "@/types/agent";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { useSessionTabs } from "@/hooks/use-session-tabs";
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
vi.mock("../agent-chat-detail", () => ({
  AgentChatDetail: (props: {
    fastModeEnabled?: boolean;
    fastModePending?: boolean;
    onFastModeChange?: (enabled: boolean) => void;
  }) => (
    <input
      type="checkbox"
      role="switch"
      aria-label="Rapide"
      checked={props.fastModeEnabled ?? false}
      disabled={props.fastModePending ?? false}
      onChange={(event) => props.onFastModeChange?.(event.target.checked)}
    />
  ),
}));
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
    filePreview: { open: false, toggleOpen: vi.fn(), openPlan: vi.fn(), openOperation: vi.fn() },
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
  return render(<AgentLocalTab navState={{ ...DEFAULT_APP_NAV.agentLocal, sessionId: "root" }} />);
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
    const cloneSwitch = screen.getByRole("switch", { name: "Rapide" });

    expect(cloneSwitch).toBeChecked();
    expect(cloneSwitch).not.toBeDisabled();
    fireEvent.click(cloneSwitch);
    expect(setFastMode).toHaveBeenCalledWith("clone", false);

    displayedSessionId = "root";
    view.rerender(<AgentLocalTab navState={{ ...DEFAULT_APP_NAV.agentLocal, sessionId: "root" }} />);
    const rootSwitch = screen.getByRole("switch", { name: "Rapide" });
    expect(rootSwitch).not.toBeChecked();
    expect(rootSwitch).not.toBeDisabled();

    fireEvent.click(rootSwitch);
    expect(setFastMode).toHaveBeenCalledWith("root", true);
  });

  it("désactive uniquement la session dont la sauvegarde est en attente", () => {
    pendingSessionId = "clone";
    renderTab();

    expect(screen.getByRole("switch", { name: "Rapide" })).toBeDisabled();
  });

  it("reflète les métadonnées confirmées après rechargement", () => {
    const view = renderTab();
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();

    displayedSessions = [rootSession, { ...cloneSession, fast_mode_enabled: false }];
    view.rerender(<AgentLocalTab navState={{ ...DEFAULT_APP_NAV.agentLocal, sessionId: "root" }} />);

    expect(screen.getByRole("switch", { name: "Rapide" })).not.toBeChecked();
  });

  it("utilise un brouillon d'accueil false indépendant des sessions", () => {
    displayedSessionId = null;
    const view = renderTab();
    const welcomeSwitch = screen.getByRole("switch", { name: "Rapide" });

    expect(welcomeSwitch).not.toBeChecked();
    fireEvent.click(welcomeSwitch);
    expect(setWelcomeFastModeEnabled).toHaveBeenCalledWith(true);

    view.rerender(<AgentLocalTab navState={{ ...DEFAULT_APP_NAV.agentLocal, sessionId: null }} />);
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeChecked();
  });
});
