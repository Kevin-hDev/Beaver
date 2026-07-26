import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { MessageList } from "../message-list";
import type { ToolActivity } from "@/hooks/agent-chat-utils";

afterEach(() => {
  cleanup();
});

vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => null,
  StreamToolTimeline: ({
    currentContent,
    currentThinking,
    currentTools,
  }: {
    currentContent: string;
    currentThinking: string;
    currentTools: ToolActivity[];
  }) => (
    <div data-testid="stream-timeline">
      {currentContent}
      {currentThinking}
      {currentTools.map((tool) => tool.name).join(",")}
    </div>
  ),
}));
vi.mock("../working-stats", () => ({
  LoadingIndicator: ({ startedAt, liveTokenCount }: { startedAt: number; liveTokenCount: number }) => (
    <div data-testid="loading-indicator">{startedAt}:{liveTokenCount}</div>
  ),
}));
vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("../context-compression-marker", () => ({ ContextCompressionMarker: () => null }));
vi.mock("../user-message", () => ({
  UserMessage: ({ content }: { content: string }) => <div data-testid="user-message">{content}</div>,
}));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../subagent-bubble", () => ({ SubagentBubble: () => null }));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("@/lib/file-preview-utils", () => ({
  collectMessageFileOperations: () => [],
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

function renderStreaming(overrides: {
  currentContent?: string;
  currentThinking?: string;
  currentTools?: ToolActivity[];
  segmentStartedAt?: number | null;
  isCompressing?: boolean;
} = {}) {
  const segmentStartedAt = "segmentStartedAt" in overrides
    ? overrides.segmentStartedAt ?? null
    : 123;

  return render(
    <MessageList
      messages={[]}
      completedSegments={[]}
      currentContent={overrides.currentContent ?? ""}
      currentThinking={overrides.currentThinking ?? ""}
      currentTools={overrides.currentTools ?? []}
      isStreaming
      isCompressing={overrides.isCompressing ?? false}
      tps={0}
      totalElapsedMs={0}
      segmentStartedAt={segmentStartedAt}
      liveTokenCount={7}
    />,
  );
}

describe("MessageList loading indicator", () => {
  it("reste visible pendant le texte live", () => {
    const view = renderStreaming({ currentContent: "texte live" });

    expect(view.getByTestId("stream-timeline").textContent).toContain("texte live");
    expect(view.getByTestId("loading-indicator").textContent).toBe("123:7");
  });

  it("reste visible pendant le thinking live", () => {
    const view = renderStreaming({ currentThinking: "thinking live" });

    expect(view.getByTestId("stream-timeline").textContent).toContain("thinking live");
    expect(view.getByTestId("loading-indicator")).toBeTruthy();
  });

  it("reste visible pendant un outil actif", () => {
    const view = renderStreaming({ currentTools: [{ name: "bash", args: {} }] });

    expect(view.getByTestId("stream-timeline").textContent).toContain("bash");
    expect(view.getByTestId("loading-indicator")).toBeTruthy();
  });

  it("laisse place à l'animation dédiée pendant la compression", () => {
    const view = renderStreaming({ currentContent: "texte live", isCompressing: true });

    expect(view.queryByTestId("loading-indicator")).toBeNull();
    expect(view.getByTestId("compression-indicator")).toBeTruthy();
  });

  it("reste masqué tant que le stream n'a pas d'heure de début", () => {
    const view = renderStreaming({ segmentStartedAt: null });

    expect(view.queryByTestId("loading-indicator")).toBeNull();
  });

  it("affiche le message en attente après le travail courant", () => {
    const view = renderStreaming({ currentContent: "travail visible" });
    view.rerender(
      <MessageList
        messages={[]}
        queuedUserMessages={[{
          id: "u2", role: "user", content: "nouvelle précision", files: [], timestamp: "2026-07-12",
        }]}
        completedSegments={[]}
        currentContent="travail visible"
        currentThinking=""
        currentTools={[]}
        isStreaming
        isCompressing={false}
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={123}
        liveTokenCount={7}
      />,
    );

    const timeline = view.getByTestId("stream-timeline");
    const user = view.getByTestId("user-message");
    expect(timeline.compareDocumentPosition(user) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(user.textContent).toBe("nouvelle précision");
  });
});
