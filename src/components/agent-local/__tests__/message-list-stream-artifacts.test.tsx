import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MessageList } from "../message-list";
import type { AgentMessage, SubagentInfo, StreamMessagePart } from "@/types/agent";

const RUN = "7c8e3a14-8811-4d88-9a54-d234547d8d22";

afterEach(cleanup);
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => <div data-testid="saved-timeline" />,
  StreamToolTimeline: () => null,
}));
vi.mock("../working-stats", () => ({ LoadingIndicator: () => null }));
vi.mock("../compression-indicator", () => ({ CompressionIndicator: () => null }));
vi.mock("../context-compression-marker", () => ({ ContextCompressionMarker: () => null }));
vi.mock("../user-message", () => ({ UserMessage: () => null }));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../subagent-bubble", () => ({
  SubagentBubble: ({ subagents }: { subagents: SubagentInfo[] }) => (
    subagents.length ? <div data-testid="subagent-bubble">{subagents[0].sessionId}</div> : null
  ),
}));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("@/lib/file-preview-utils", () => ({ collectFileOperations: () => [] }));

describe("MessageList stream artifacts", () => {
  it("masque le récapitulatif du groupe actif", () => {
    const { queryByTestId } = renderList([grouped(delegate(), "checkpoint")], true);
    expect(queryByTestId("subagent-bubble")).toBeNull();
  });

  it("affiche un seul récapitulatif sous la réponse finale", () => {
    const checkpoint = grouped(delegate(), "checkpoint");
    const input = grouped(base("input", "user"), "input");
    const final = grouped(base("final", "assistant"), "final");
    const { getAllByTestId } = renderList([checkpoint, input, final], false);
    const timelines = getAllByTestId("saved-timeline");
    const bubbles = getAllByTestId("subagent-bubble");
    expect(bubbles).toHaveLength(1);
    expect(timelines[1].compareDocumentPosition(bubbles[0]))
      .toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });
});

function renderList(messages: AgentMessage[], isStreaming: boolean) {
  return render(<MessageList
    messages={messages} completedSegments={[]}
    currentContent="" currentThinking="" currentTools={[]} isStreaming={isStreaming}
    isCompressing={false}
    streamRunId={RUN} tps={0} totalElapsedMs={0} segmentStartedAt={null}
    liveTokenCount={0} knownSubagents={[known()]}
  />);
}

function grouped(message: AgentMessage, stream_part: StreamMessagePart): AgentMessage {
  return { ...message, stream_run_id: RUN, stream_part };
}

function delegate(): AgentMessage {
  const message = base("checkpoint", "assistant");
  message.segments = [{ content: "", phase: "work", tools: [{
    name: "delegate_task", summary: "delegate", args: { subagent_type: "explorer" },
    result: '<subagent id="child-current" state="running">ok</subagent>',
  }] }];
  return message;
}

function base(id: string, role: AgentMessage["role"]): AgentMessage {
  return { id, role, content: id, files: [], timestamp: "2026-07-12T10:00:00Z",
    segments: role === "assistant" ? [{ content: id, tools: [], phase: "final" }] : undefined };
}

function known(): SubagentInfo {
  return { sessionId: "child-current", name: "Audit", type: "explorer",
    status: "completed", promptPreview: "" };
}
