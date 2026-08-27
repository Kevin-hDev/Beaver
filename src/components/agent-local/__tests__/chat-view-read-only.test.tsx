import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatView } from "../chat-view";
import type { ChatViewProps } from "../chat-view-types";
import type { AgentMessage } from "@/types/agent";

const state = vi.hoisted(() => ({
  chat: {
    messages: [] as AgentMessage[], sessionTokenCount: 0, contextLimitTokens: 0, planModeEnabled: false,
    completedSegments: [], currentTools: [], currentContent: "", currentContentPhase: undefined,
    currentThinking: "", isStreaming: false, planPreview: null, sessionLoading: false,
    contextUsageVisible: false, forbiddenAllowedPaths: [], dismissForbiddenDirectory: vi.fn(),
    error: "Stream failed", isConnectionError: false, diagnosticSummary: undefined,
  },
  runtime: { showError: true, handleRetry: vi.fn() },
  errorBubbleProps: undefined as { onRetry?: () => void } | undefined,
  dropZoneProps: undefined as { enabled?: boolean } | undefined,
  overlayProps: undefined as { readOnly?: boolean } | undefined,
  runtimeParams: undefined as { readOnly?: boolean } | undefined,
  chatActionsOptions: undefined as { readOnly?: boolean } | undefined,
  permissionModeEnabled: undefined as boolean | undefined,
  isAtBottom: false,
}));

vi.mock("../chat-message-panel", () => ({
  ChatMessagePanel: ({ activeSearchMessageId }: { activeSearchMessageId?: string | null }) => (
    <div data-testid="message-panel">
      {state.chat.messages.map((message) => (
        <div
          key={message.id}
          data-message-id={message.id}
          className={activeSearchMessageId === message.id ? "cfs-match-active" : ""}
        >
          {message.content}
        </div>
      ))}
    </div>
  ),
}));
vi.mock("../chat-input", () => ({ ChatInput: () => <div data-testid="chat-input" /> }));
vi.mock("../chat-input-footer", () => ({ ChatInputFooter: () => <div data-testid="chat-input-footer" /> }));
vi.mock("../scroll-bottom-button", () => ({ ScrollBottomButton: () => <div data-testid="scroll-bottom" /> }));
vi.mock("../chat-terminal-dock", () => ({ ChatTerminalDock: () => <div data-testid="terminal-dock" /> }));
vi.mock("../todo-progress-panel", () => ({ TodoProgressPanel: () => <div data-testid="todo-panel" /> }));
vi.mock("../subagent-accordion", () => ({ SubagentAccordion: () => <div data-testid="subagent-accordion" /> }));
vi.mock("../permission-dialog", () => ({ PermissionDialog: () => <div data-testid="permission-dialog" /> }));
vi.mock("../error-bubble", () => ({
  ErrorBubble: (props: { onRetry?: () => void }) => {
    state.errorBubbleProps = props;
    return <div data-testid="stream-error" />;
  },
}));
vi.mock("../file-drop-zone", () => ({
  FileDropZone: (props: { children: ReactNode; enabled?: boolean }) => {
    state.dropZoneProps = props;
    return <>{props.children}</>;
  },
}));
vi.mock("../chat-overlays", () => ({
  ChatOverlays: (props: { readOnly?: boolean }) => {
    state.overlayProps = props;
    return null;
  },
}));
vi.mock("@/hooks/use-agent-chat", () => ({ useAgentChat: () => state.chat }));
vi.mock("@/hooks/use-context-progress", () => ({ useContextProgress: () => ({ max: 0 }) }));
vi.mock("@/hooks/use-context-usage", () => ({ useContextUsage: () => ({ used: 0 }) }));
vi.mock("@/hooks/use-file-drop", () => ({ useFileDrop: () => ({ dragging: false, setDragging: vi.fn(), addByPaths: vi.fn(), files: [] }) }));
vi.mock("@/hooks/use-permission-mode", () => ({
  usePermissionMode: (_sessionId: string, enabled: boolean) => {
    state.permissionModeEnabled = enabled;
    return { mode: "auto", refresh: vi.fn(), availableModes: [] };
  },
}));
vi.mock("@/hooks/use-permission-requests", () => ({ usePermissionRequests: () => ({ enqueue: vi.fn(), current: { id: "request" }, respond: vi.fn() }) }));
vi.mock("@/hooks/use-session-project", () => ({ useSessionProject: () => ({ selectedProject: undefined, selectedProjectId: undefined, directoryAccessPrompt: undefined }) }));
vi.mock("@/hooks/use-chat-scroll", () => ({ useChatScroll: () => ({ containerRef: { current: null }, isAtBottom: state.isAtBottom, scrollToBottom: vi.fn() }) }));
vi.mock("@/hooks/use-model-switch", () => ({ useModelSwitch: () => ({ pendingSwitch: null, setPendingSwitch: vi.fn(), handleModelSelect: vi.fn(), rememberedRef: { current: null } }) }));
vi.mock("@/hooks/use-worktree-session-switch", () => ({ useWorktreeSessionSwitch: () => ({ pending: null, request: vi.fn(), cancel: vi.fn(), createSession: vi.fn() }) }));
vi.mock("@/hooks/use-session-files", () => ({ useSessionFileGroups: vi.fn() }));
vi.mock("@/hooks/use-subagents", () => ({ useSubagents: () => ({ active: [{ id: "child" }], completed: [] }) }));
vi.mock("@/hooks/use-chat-actions", () => ({
  useChatActions: (options: { readOnly?: boolean }) => {
    state.chatActionsOptions = options;
    return { handleSend: vi.fn(), handleFileImport: vi.fn() };
  },
}));
vi.mock("@/hooks/use-chat-clone", () => ({ useChatClone: () => ({ requestClone: vi.fn(), summaryRun: null, pendingClone: null, cloneBusy: false, cancelClone: vi.fn(), abortClone: vi.fn(), submitClone: vi.fn() }) }));
vi.mock("@/hooks/use-clone-git-branch-action", () => ({ useCloneGitBranchAction: () => undefined }));
vi.mock("@/hooks/use-selected-model-capabilities", () => ({ useSelectedModelCapabilities: () => undefined }));
vi.mock("@/hooks/use-chat-view-runtime", () => ({
  useChatViewRuntime: (params: { readOnly?: boolean }) => {
    state.runtimeParams = params;
    return state.runtime;
  },
}));
vi.mock("@/hooks/use-preflight-directory-access-prompt", () => ({ usePreflightDirectoryAccessPrompt: () => undefined }));
vi.mock("@/hooks/use-composer-handoff", () => ({ useComposerHandoff: vi.fn() }));
vi.mock("@/lib/composer-handoff", () => ({ hasComposerPosition: () => true }));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

const props = {
  sessionId: "parent", model: "model", provider: "provider", projects: [], git: {},
  onAddProject: vi.fn(), onReasoningModeChange: vi.fn(), terminalState: {},
} as unknown as ChatViewProps;
const originalScrollIntoView = Object.getOwnPropertyDescriptor(
  Element.prototype,
  "scrollIntoView",
);

afterEach(() => {
  state.chat.messages = [];
  state.isAtBottom = false;
  if (originalScrollIntoView) {
    Object.defineProperty(Element.prototype, "scrollIntoView", originalScrollIntoView);
  } else {
    Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  }
});

describe("ChatView child read-only mode", () => {
  it.each([
    ["Ctrl", { code: "KeyF", key: "f", ctrlKey: true }],
    ["Cmd", { code: "KeyF", key: "f", metaKey: true }],
  ])("ouvre la recherche dans la conversation avec %s + F", (_label, keyboard) => {
    render(<ChatView {...props} isSubagent={false} />);

    fireEvent.keyDown(window, keyboard);

    expect(screen.getByRole("searchbox")).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("searchbox")).not.toBeInTheDocument();
  });

  it("parcourt les messages correspondants avec Entrée et Maj + Entrée", () => {
    state.chat.messages = [
      message("first", "Première aiguille"),
      message("middle", "Sans correspondance"),
      message("last", "Deuxième AIGUILLE"),
    ];
    const scrollIntoView = vi.fn();
    Object.defineProperty(Element.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    render(<ChatView {...props} isSubagent={false} />);
    fireEvent.keyDown(window, { code: "KeyF", key: "f", ctrlKey: true });

    const input = screen.getByRole("searchbox");
    fireEvent.change(input, { target: { value: "aiguille" } });

    expect(screen.getByText("1 / 2")).toBeInTheDocument();
    expect(document.querySelector("[data-message-id='first']")).toHaveClass("cfs-match-active");
    expect(scrollIntoView).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(input, { key: "Enter" });
    expect(screen.getByText("2 / 2")).toBeInTheDocument();
    expect(document.querySelector("[data-message-id='last']")).toHaveClass("cfs-match-active");

    fireEvent.keyDown(input, { key: "Enter", shiftKey: true });
    expect(screen.getByText("1 / 2")).toBeInTheDocument();
    expect(document.querySelector("[data-message-id='first']")).toHaveClass("cfs-match-active");
  });

  it("removes write surfaces when rerendered from a parent without unmounting", () => {
    const { rerender } = render(<ChatView {...props} isSubagent={false} />);

    expect(screen.getByTestId("chat-input")).toBeInTheDocument();
    expect(screen.getByTestId("chat-input-footer")).toBeInTheDocument();
    expect(screen.getByTestId("terminal-dock")).toBeInTheDocument();
    expect(screen.getByTestId("todo-panel")).toBeInTheDocument();
    expect(screen.getByTestId("permission-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("subagent-accordion")).toBeInTheDocument();
    expect(screen.getByTestId("scroll-bottom")).toBeInTheDocument();
    expect(state.errorBubbleProps?.onRetry).toEqual(expect.any(Function));
    expect(state.dropZoneProps?.enabled).toBe(true);
    expect(state.overlayProps?.readOnly).toBe(false);
    expect(state.runtimeParams?.readOnly).toBe(false);
    expect(state.chatActionsOptions?.readOnly).toBe(false);
    expect(state.permissionModeEnabled).toBe(true);

    rerender(<ChatView {...props} isSubagent />);

    expect(document.querySelector(".chat-zone-read-only")).toBeInTheDocument();
    expect(screen.getByTestId("message-panel")).toBeInTheDocument();
    expect(screen.getByTestId("stream-error")).toBeInTheDocument();
    expect(state.errorBubbleProps?.onRetry).toBeUndefined();
    expect(screen.queryByTestId("chat-input")).not.toBeInTheDocument();
    expect(screen.queryByTestId("chat-input-footer")).not.toBeInTheDocument();
    expect(screen.queryByTestId("terminal-dock")).not.toBeInTheDocument();
    expect(screen.queryByTestId("todo-panel")).not.toBeInTheDocument();
    expect(screen.queryByTestId("permission-dialog")).not.toBeInTheDocument();
    expect(screen.queryByTestId("subagent-accordion")).not.toBeInTheDocument();
    expect(screen.getByTestId("scroll-bottom")).toBeInTheDocument();
    expect(state.dropZoneProps?.enabled).toBe(false);
    expect(state.overlayProps?.readOnly).toBe(true);
    expect(state.runtimeParams?.readOnly).toBe(true);
    expect(state.chatActionsOptions?.readOnly).toBe(true);
    expect(state.permissionModeEnabled).toBe(false);

    state.isAtBottom = true;
    rerender(<ChatView {...props} isSubagent />);

    expect(document.querySelector(".chat-read-only-footer")).toBeInTheDocument();
    expect(screen.queryByTestId("scroll-bottom")).not.toBeInTheDocument();
  });
});

function message(id: string, content: string): AgentMessage {
  return {
    id,
    role: "user",
    content,
    files: [],
    timestamp: "2026-08-27T00:00:00Z",
  };
}
