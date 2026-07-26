import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MessageList } from "../message-list";
import type { AgentMessage, SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());
vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: ({ segments }: { segments: AgentMessage["segments"] }) => (
    <div data-testid="saved-timeline">{JSON.stringify(segments)}</div>
  ),
  StreamToolTimeline: () => null,
}));
vi.mock("../working-stats", () => ({ LoadingIndicator: () => null }));
vi.mock("../compression-indicator", () => ({ CompressionIndicator: () => null }));
vi.mock("../context-compression-marker", () => ({ ContextCompressionMarker: () => null }));
vi.mock("../user-message", () => ({ UserMessage: () => null }));
vi.mock("../assistant-message", () => ({ AssistantMessage: () => null }));
vi.mock("../subagent-bubble", () => ({
  SubagentBubble: ({ subagents }: { subagents: SubagentInfo[] }) => (
    <div data-testid="subagent-bubble">
      {subagents.map((agent) => `${agent.sessionId}:${agent.name}`).join(",")}
    </div>
  ),
}));
vi.mock("../plan-preview-bubble", () => ({ PlanPreviewBubble: () => null }));
vi.mock("../file-change-bubble", () => ({ FileChangeBubble: () => null }));
vi.mock("@/lib/file-preview-utils", () => ({ collectFileOperations: () => [] }));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));

const messages: AgentMessage[] = [{
  id: "u1",
  role: "user",
  content: "Question",
  timestamp: "2026-07-07T10:00:00Z",
  files: [],
}];

describe("MessageList subagents", () => {
  it("affiche l'historique technique enfant avec la timeline du parent", () => {
    const rawHistory: AgentMessage[] = [
      ...messages,
      {
        id: "work", role: "assistant", content: "Je vérifie", thinking: "Analyse",
        tool_calls: [{ function: { name: "read_file", arguments: { path: "README.md" } } }],
        timestamp: "2026-07-07T10:00:01Z", files: [],
      },
      {
        id: "tool", role: "tool", content: "Contenu du projet", tool_name: "read_file",
        timestamp: "2026-07-07T10:00:02Z", files: [],
      },
      {
        id: "final", role: "assistant", content: "Rapport final",
        timestamp: "2026-07-07T10:00:03Z", files: [],
      },
    ];

    const { getAllByTestId } = render(
      <MessageList
        messages={rawHistory} completedSegments={[]}
        currentContent="" currentThinking="" currentTools={[]} isStreaming={false}
        isCompressing={false}
        tps={0} totalElapsedMs={0} segmentStartedAt={null} liveTokenCount={0}
      />,
    );

    const timelines = getAllByTestId("saved-timeline");
    expect(timelines).toHaveLength(1);
    expect(timelines[0]).toHaveTextContent("Analyse");
    expect(timelines[0]).toHaveTextContent("read_file");
    expect(timelines[0]).toHaveTextContent("Contenu du projet");
    expect(timelines[0]).toHaveTextContent("Rapport final");
  });

  it("affiche uniquement les sous-agents créés par le message sauvegardé", () => {
    const { queryByTestId } = render(
      <MessageList
        messages={[...messages, assistantWithDelegate("child-current")]}
        completedSegments={[]}
        currentContent=""
        currentThinking=""
        currentTools={[]}
        isStreaming={false}
        isCompressing={false}
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={null}
        liveTokenCount={0}
        knownSubagents={[{
          sessionId: "child-current",
          name: "Audit courant",
          type: "explorer",
          status: "completed",
          promptPreview: "",
        }, {
          sessionId: "child-old",
          name: "Ancien audit",
          type: "coder",
          status: "completed",
          promptPreview: "",
        }]}
      />,
    );

    expect(queryByTestId("subagent-bubble")).toHaveTextContent("child-current:Audit courant");
    expect(queryByTestId("subagent-bubble")).not.toHaveTextContent("child-old");
  });

  it("place la bubble sous la réponse finale sauvegardée", () => {
    render(
      <MessageList
        messages={[assistantWithDelegate("child-current")]}
        completedSegments={[]}
        currentContent=""
        currentThinking=""
        currentTools={[]}
        isStreaming={false}
        isCompressing={false}
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={null}
        liveTokenCount={0}
        knownSubagents={[{
          sessionId: "child-current",
          name: "Audit courant",
          type: "explorer",
          status: "completed",
          promptPreview: "",
        }]}
      />,
    );

    const timeline = renderedTimeline();
    const bubble = renderedBubble();
    expect(timeline.compareDocumentPosition(bubble)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });

});

function renderedBubble() {
  return document.querySelector("[data-testid='subagent-bubble']") as HTMLElement;
}

function renderedTimeline() {
  return document.querySelector("[data-testid='saved-timeline']") as HTMLElement;
}

function assistantWithDelegate(sessionId: string): AgentMessage {
  return {
    id: `assistant-${sessionId}`,
    role: "assistant",
    content: "Réponse finale",
    files: [],
    timestamp: "2026-07-07T10:01:00Z",
    segments: [
      {
        content: "",
        tools: [{
          name: "delegate_task",
          summary: "delegate",
          args: { subagent_type: "explorer", prompt: "Audit" },
          result: `<subagent id="${sessionId}" state="running">ok</subagent>`,
        }],
        phase: "work",
      },
      {
        content: "Réponse finale",
        tools: [],
        phase: "final",
      },
    ],
  };
}
