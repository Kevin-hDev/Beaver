import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { MessageList } from "../message-list";
import type { AgentMessage } from "@/types/agent";

afterEach(() => {
  cleanup();
});

vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("../context-compression-marker", () => ({
  ContextCompressionMarker: () => <div data-testid="context-marker" />,
}));
vi.mock("../user-message", () => ({ UserMessage: () => null }));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => null,
  StreamToolTimeline: () => null,
}));
vi.mock("../subagent-bubble", () => ({ SubagentBubble: () => null }));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("../working-stats", () => ({
  LoadingIndicator: () => <div data-testid="loading-indicator" />,
}));
vi.mock("@/lib/file-preview-utils", () => ({
  collectMessageFileOperations: () => [],
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

function renderList(messages: AgentMessage[], isCompressing = false) {
  return render(
    <MessageList
      messages={messages}
      completedSegments={[]}
      currentContent=""
      currentThinking=""
      currentTools={[]}
      isStreaming={isCompressing}
      isCompressing={isCompressing}
      tps={0}
      totalElapsedMs={0}
      segmentStartedAt={isCompressing ? 123 : null}
      liveTokenCount={0}
    />,
  );
}

describe("MessageList compression indicator", () => {
  it("affiche seulement l'animation dédiée pendant une nouvelle compression", () => {
    const compressedMessage: AgentMessage = {
      id: "compressed-1",
      role: "assistant",
      content: "This session is being continued from a previous conversation.",
      files: [],
      timestamp: new Date(0).toISOString(),
    };

    const view = renderList([compressedMessage], true);

    expect(view.getByTestId("compression-indicator")).toBeTruthy();
    expect(view.queryByTestId("loading-indicator")).toBeNull();
    expect(view.getByTestId("context-marker")).toBeTruthy();
  });
});
