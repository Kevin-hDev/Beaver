import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { ChatView } from "../chat-view";
import type { ChatViewProps } from "../chat-view-types";

const state = vi.hoisted(() => ({
  chat: {
    messages: [], sessionTokenCount: 0, contextLimitTokens: 0, planModeEnabled: false,
    completedSegments: [], currentTools: [], currentContent: "", currentContentPhase: undefined,
    currentThinking: "", isStreaming: false, planPreview: null, sessionLoading: false,
    contextUsageVisible: false, forbiddenAllowedPaths: [], dismissForbiddenDirectory: vi.fn(),
    error: "Stream failed", isConnectionError: false, diagnosticSummary: undefined,
  },
  runtime: { showError: true, handleRetry: vi.fn() },
  errorBubbleProps: undefined as { onRetry?: () => void } | undefined,
}));

vi.mock("../chat-message-panel", () => ({ ChatMessagePanel: () => <div data-testid="message-panel" /> }));
vi.mock("../chat-input", () => ({ ChatInput: () => <div data-testid="chat-input" /> }));
vi.mock("../chat-input-footer", () => ({ ChatInputFooter: () => <div data-testid="chat-input-footer" /> }));
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
vi.mock("../file-drop-zone", () => ({ FileDropZone: ({ children }: { children: ReactNode }) => <>{children}</> }));
vi.mock("../chat-overlays", () => ({ ChatOverlays: () => null }));
vi.mock("@/hooks/use-agent-chat", () => ({ useAgentChat: () => state.chat }));
vi.mock("@/hooks/use-context-progress", () => ({ useContextProgress: () => ({ max: 0 }) }));
vi.mock("@/hooks/use-context-usage", () => ({ useContextUsage: () => ({ used: 0 }) }));
vi.mock("@/hooks/use-file-drop", () => ({ useFileDrop: () => ({ dragging: false, setDragging: vi.fn(), addByPaths: vi.fn(), files: [] }) }));
vi.mock("@/hooks/use-permission-mode", () => ({ usePermissionMode: () => ({ mode: "auto", refresh: vi.fn(), availableModes: [] }) }));
vi.mock("@/hooks/use-permission-requests", () => ({ usePermissionRequests: () => ({ enqueue: vi.fn(), current: { id: "request" }, respond: vi.fn() }) }));
vi.mock("@/hooks/use-session-project", () => ({ useSessionProject: () => ({ selectedProject: undefined, selectedProjectId: undefined, directoryAccessPrompt: undefined }) }));
vi.mock("@/hooks/use-chat-scroll", () => ({ useChatScroll: () => ({ containerRef: { current: null }, isAtBottom: true, scrollToBottom: vi.fn() }) }));
vi.mock("@/hooks/use-model-switch", () => ({ useModelSwitch: () => ({ pendingSwitch: null, setPendingSwitch: vi.fn(), handleModelSelect: vi.fn(), rememberedRef: { current: null } }) }));
vi.mock("@/hooks/use-worktree-session-switch", () => ({ useWorktreeSessionSwitch: () => ({ pending: null, request: vi.fn(), cancel: vi.fn(), createSession: vi.fn() }) }));
vi.mock("@/hooks/use-session-files", () => ({ useSessionFileGroups: vi.fn() }));
vi.mock("@/hooks/use-subagents", () => ({ useSubagents: () => ({ active: [{ id: "child" }], completed: [] }) }));
vi.mock("@/hooks/use-chat-actions", () => ({ useChatActions: () => ({ handleSend: vi.fn(), handleFileImport: vi.fn() }) }));
vi.mock("@/hooks/use-chat-clone", () => ({ useChatClone: () => ({ requestClone: vi.fn(), summaryRun: null, pendingClone: null, cloneBusy: false, cancelClone: vi.fn(), abortClone: vi.fn(), submitClone: vi.fn() }) }));
vi.mock("@/hooks/use-clone-git-branch-action", () => ({ useCloneGitBranchAction: () => undefined }));
vi.mock("@/hooks/use-selected-model-capabilities", () => ({ useSelectedModelCapabilities: () => undefined }));
vi.mock("@/hooks/use-chat-view-runtime", () => ({ useChatViewRuntime: () => state.runtime }));
vi.mock("@/hooks/use-preflight-directory-access-prompt", () => ({ usePreflightDirectoryAccessPrompt: () => undefined }));
vi.mock("@/hooks/use-composer-handoff", () => ({ useComposerHandoff: vi.fn() }));
vi.mock("@/lib/composer-handoff", () => ({ hasComposerPosition: () => true }));

const props = {
  sessionId: "parent", model: "model", provider: "provider", projects: [], git: {},
  onAddProject: vi.fn(), onReasoningModeChange: vi.fn(), terminalState: {},
} as unknown as ChatViewProps;

describe("ChatView child read-only mode", () => {
  it("removes write surfaces when rerendered from a parent without unmounting", () => {
    const { rerender } = render(<ChatView {...props} isSubagent={false} />);

    expect(screen.getByTestId("chat-input")).toBeInTheDocument();
    expect(screen.getByTestId("chat-input-footer")).toBeInTheDocument();
    expect(screen.getByTestId("terminal-dock")).toBeInTheDocument();
    expect(screen.getByTestId("todo-panel")).toBeInTheDocument();
    expect(screen.getByTestId("permission-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("subagent-accordion")).toBeInTheDocument();
    expect(state.errorBubbleProps?.onRetry).toEqual(expect.any(Function));

    rerender(<ChatView {...props} isSubagent />);

    expect(screen.getByTestId("message-panel")).toBeInTheDocument();
    expect(screen.getByTestId("stream-error")).toBeInTheDocument();
    expect(state.errorBubbleProps?.onRetry).toBeUndefined();
    expect(screen.queryByTestId("chat-input")).not.toBeInTheDocument();
    expect(screen.queryByTestId("chat-input-footer")).not.toBeInTheDocument();
    expect(screen.queryByTestId("terminal-dock")).not.toBeInTheDocument();
    expect(screen.queryByTestId("todo-panel")).not.toBeInTheDocument();
    expect(screen.queryByTestId("permission-dialog")).not.toBeInTheDocument();
    expect(screen.queryByTestId("subagent-accordion")).not.toBeInTheDocument();
  });
});
