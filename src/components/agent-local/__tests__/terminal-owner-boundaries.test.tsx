/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { AgentLocalTab } from "../agent-local-tab";

const gitDoubles = vi.hoisted(() => ({ dialogs: null as ReactNode }));

vi.mock("@/hooks/use-agent-local-tab", () => ({ useAgentLocalTab: vi.fn() }));
vi.mock("@/hooks/use-session-tabs", () => ({
  useSessionTabs: () => ({
    activeSessionId: null, activeTab: null, tabs: [], attentionTabIds: [],
    renameTab: vi.fn(), cloneMessage: vi.fn(), cancelCloneSummary: vi.fn(),
    createCloneGitBranch: vi.fn(), linkCloneGitBranch: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-file-tree", () => ({ useFileTree: () => ({}) }));
vi.mock("@/hooks/use-forecast-panel", () => ({ useForecastPanel: () => ({}) }));
vi.mock("@/hooks/use-agent-local-panel-nav", () => ({ useAgentLocalPanelNav: vi.fn() }));
vi.mock("@/hooks/use-agent-local-controlled-panels", () => ({
  useAgentLocalControlledPanels: () => ({
    fileTreeNav: {}, forecastNav: { panelMode: "preview", setPanelMode: vi.fn() },
  }),
}));
vi.mock("@/hooks/use-git-branch", () => ({ useGitBranch: () => ({}) }));
vi.mock("@/hooks/use-git-uncommitted-files", () => ({ useGitUncommittedFiles: () => [] }));
vi.mock("@/hooks/use-session-summary", () => ({ useSessionSummary: () => ({}) }));
vi.mock("@/hooks/use-agent-local-tab-git", () => ({
  useAgentLocalTabGit: () => ({ selectTab: vi.fn(), closeTab: vi.fn(), dialogs: gitDoubles.dialogs }),
}));
vi.mock("../use-agent-local-forecast-content", () => ({
  useAgentLocalForecastContent: () => ({
    forecastContent: null, fullscreenSwitching: false,
    handleOpenForecastDocs: vi.fn(), handlePreviewFullscreenChange: vi.fn(),
  }),
}));
vi.mock("../use-agent-local-conversation-list", () => ({
  useAgentLocalConversationList: () => null,
}));
vi.mock("@/hooks/use-available-panel-mode", () => ({
  useAvailablePanelMode: () => ({ panelMode: "preview", browserStatus: "unavailable" }),
}));
vi.mock("@/components/ui/panel-slots", () => ({
  PanelSlot: ({ children }: { children: ReactNode }) => <>{children}</>,
}));
vi.mock("../chat-header", () => ({ ChatHeader: () => null }));
vi.mock("../welcome-view", () => ({
  WelcomeView: () => <div data-testid="welcome-view" />,
}));
vi.mock("../agent-chat-detail", () => ({
  AgentChatDetail: () => <div data-testid="agent-chat-detail" />,
}));

function stateFor(sessionId: string | null) {
  const session = sessionId ? { id: sessionId, name: "Session" } : null;
  return {
    sessions: session ? [session] : [],
    refresh: vi.fn(), updateModel: vi.fn(),
    projectsHook: { projects: [], add: vi.fn() },
    terminal: { isOpen: false, tabs: [], togglePanel: vi.fn() },
    activeSession: session, activeSessionId: sessionId,
    model: "model", provider: "provider", currentDefault: { model: "model", provider: "provider" },
    activeProject: null, filePreview: {
      open: false, toggleOpen: vi.fn(), openPlan: vi.fn(), openOperation: vi.fn(),
    },
    fileOperations: { all: [] }, setFileOperations: vi.fn(),
    reasoningMode: null, setReasoningMode: vi.fn(), setWelcomeModel: vi.fn(),
    welcomeFastModeEnabled: false, setWelcomeFastModeEnabled: vi.fn(),
    setFastMode: vi.fn(), isFastModePending: () => false,
    sessionActions: {
      handleCreateWithModel: vi.fn(), handleWelcomeSend: vi.fn(),
      handleAutoRename: vi.fn(), handleCreateInProjectWithModel: vi.fn(),
    }, handleSelectById: vi.fn(), handleArchiveSession: vi.fn(),
  };
}

describe("frontières de propriété Agent Local", () => {
  beforeEach(() => {
    vi.mocked(useAgentLocalTab).mockImplementation(({ navState }) => stateFor(navState.sessionId) as never);
  });

  afterEach(() => {
    cleanup();
    gitDoubles.dialogs = null;
    vi.clearAllMocks();
  });

  it("démonte le détail au retour vers l'accueil sans démonter le propriétaire", () => {
    const view = (sessionId: string | null) => (
      <AgentLocalTab navState={{ ...DEFAULT_AGENT_LOCAL_NAV, sessionId }} />
    );
    const rendered = render(view("session-1"));
    expect(screen.getByTestId("agent-chat-detail")).toBeTruthy();
    expect(screen.queryByTestId("welcome-view")).toBeNull();
    expect(document.querySelectorAll(".al-panel-surface")).toHaveLength(2);
    expect(document.querySelector(".al-overlay-surface")).toBeTruthy();

    rendered.rerender(view(null));
    expect(screen.queryByTestId("agent-chat-detail")).toBeNull();
    expect(screen.getByTestId("welcome-view")).toBeTruthy();
  });

  it("conserve le texte du conflit Git monté pendant l'inactivité", () => {
    gitDoubles.dialogs = (
      <div data-testid="branch-conflict">
        Conflit de branche
        <input aria-label="description conflit" defaultValue="" />
      </div>
    );
    const view = (active: boolean) => (
      <AppSurfaceActivityProvider active={active}>
        <AgentLocalTab navState={{ ...DEFAULT_AGENT_LOCAL_NAV, sessionId: null }} />
      </AppSurfaceActivityProvider>
    );
    const rendered = render(view(true));
    expect(screen.getByTestId("branch-conflict")).toHaveTextContent("Conflit de branche");
    fireEvent.change(screen.getByRole("textbox", { name: "description conflit" }), {
      target: { value: "details conservés" },
    });

    rendered.rerender(view(false));
    expect(screen.getByTestId("branch-conflict")).toHaveTextContent("Conflit de branche");
    expect(screen.getByTestId("branch-conflict").querySelector("input"))
      .toHaveValue("details conservés");
    expect(screen.getByTestId("branch-conflict").closest(".al-overlay-surface"))
      .toHaveAttribute("hidden");

    rendered.rerender(view(true));
    expect(screen.getByTestId("branch-conflict")).toHaveTextContent("Conflit de branche");
    expect(screen.getByRole("textbox", { name: "description conflit" })).toHaveValue("details conservés");
  });
});
