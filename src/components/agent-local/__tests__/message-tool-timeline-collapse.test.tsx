import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { SavedToolTimeline, StreamToolTimeline } from "../message-tool-timeline";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span data-testid="caret-down" />,
  CaretRight: () => <span data-testid="caret-right" />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const duration = opts?.duration;
      const durationText = typeof duration === "string" || typeof duration === "number" ? String(duration) : "";
      if (key === "agentLocal.workSummary") return `Worked for ${durationText}`;
      if (key === "agentLocal.workSummaryNoDuration") return "Work completed";
      return key;
    },
  }),
}));
vi.mock("../assistant-message", () => ({
  AssistantMessage: ({
    content,
    isStreaming,
    thinking,
    totalElapsedMs,
    showActions = true,
  }: {
    content: string;
    isStreaming?: boolean;
    thinking?: string;
    totalElapsedMs?: number;
    showActions?: boolean;
  }) => (
    <div
      data-testid={isStreaming ? "assistant-stream" : "assistant"}
      data-elapsed={totalElapsedMs ?? ""}
      data-actions={showActions ? "true" : "false"}
    >
      {thinking && <span>{thinking}</span>}
      {content}
    </div>
  ),
}));
vi.mock("../thinking-section", () => ({
  ThinkingSection: ({ content }: { content: string }) => <div data-testid="thinking">{content}</div>,
}));
vi.mock("../tool-bubble", () => ({
  ToolBubble: ({ tools }: { tools: Array<{ name: string }> }) => (
    <div data-testid="tool-bubble">{tools.map((tool) => tool.name).join(",")}</div>
  ),
  SavedToolBubble: ({ tools }: { tools: Array<{ name: string }> }) => (
    <div data-testid="saved-tool-bubble">{tools.map((tool) => tool.name).join(",")}</div>
  ),
}));
vi.mock("../branch-bubble", () => ({
  BranchBubble: () => <div data-testid="branch-bubble" />,
}));

describe("message tool timeline collapse", () => {
  it("garde la phase de travail visible tant que la réponse finale n'a pas commencé", () => {
    const { getByText, queryByText } = render(
      <StreamToolTimeline
        completedSegments={[]}
        currentThinking="thinking live"
        currentContent=""
        currentTools={[{ name: "bash", args: {} }]}
        streamStartedAt={1}
      />,
    );

    expect(getByText("thinking live")).toBeTruthy();
    expect(getByText("bash")).toBeTruthy();
    expect(queryByText(/Worked for/)).toBeNull();
  });

  it("replie la phase de travail dès que la réponse finale commence", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-01T12:04:26Z"));
    const startedAt = Date.now() - 266_000;
    const { getByRole, getByText, queryByText } = render(
      <StreamToolTimeline
        completedSegments={[{
          thinking: "thinking before final",
          content: "intermediate note",
          tools: [{ name: "bash", args: {}, result: "ok" }],
        }]}
        currentThinking=""
        currentContent="final answer"
        currentContentPhase="final"
        currentTools={[]}
        streamStartedAt={startedAt}
      />,
    );

    const toggle = getByRole("button", { name: /Worked for 4 min 26 s/ });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(getByText("final answer")).toBeTruthy();
    expect(queryByText("intermediate note")).toBeNull();

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(getByText("intermediate note")).toBeTruthy();
    expect(getByText("thinking before final")).toBeTruthy();
    expect(getByText("bash")).toBeTruthy();
  });

  it("ne replie pas un texte de travail même après un ancien bloc de tools", () => {
    const { getByText, queryByText } = render(
      <StreamToolTimeline
        completedSegments={[{
          thinking: "thinking before work",
          content: "",
          tools: [{ name: "bash", args: {}, result: "ok" }],
        }]}
        currentThinking=""
        currentContent="intermediate work note"
        currentContentPhase="work"
        currentTools={[]}
        streamStartedAt={1}
      />,
    );

    expect(getByText("intermediate work note")).toBeTruthy();
    expect(queryByText(/Worked for|Work completed/)).toBeNull();
  });

  it("ne replie pas un texte live tant que le backend n'a pas confirmé la phase finale", () => {
    const { getByText, queryByText } = render(
      <StreamToolTimeline
        completedSegments={[{
          thinking: "thinking before pending",
          content: "",
          tools: [{ name: "bash", args: {}, result: "ok" }],
        }]}
        currentThinking=""
        currentContent="pending live text"
        currentTools={[]}
        streamStartedAt={1}
      />,
    );

    expect(getByText("pending live text")).toBeTruthy();
    expect(queryByText(/Worked for|Work completed/)).toBeNull();
  });

  it("replie les segments sauvegardés par défaut et laisse la réponse finale visible", () => {
    const { getByRole, getByTestId, getByText, queryByText } = render(
      <SavedToolTimeline
        messageId="m1"
        segments={[
          { thinking: "saved thinking", content: "saved intermediate", tools: [{ name: "grep", summary: "x" }] },
          { content: "saved final", tools: [] },
        ]}
        tps={0}
        totalElapsedMs={125_000}
      />,
    );

    const toggle = getByRole("button", { name: /Worked for 2 min 5 s/ });
    expect(getByText("saved final")).toBeTruthy();
    expect(getByTestId("assistant")).toHaveAttribute("data-elapsed", "");
    expect(queryByText("saved intermediate")).toBeNull();

    fireEvent.click(toggle);
    expect(getByText("saved intermediate")).toBeTruthy();
    expect(getByText("saved intermediate").closest("[data-testid='assistant']")).toHaveAttribute("data-actions", "false");
    expect(getByText("saved thinking")).toBeTruthy();
    expect(getByText("grep")).toBeTruthy();
  });

  it("replie un stream sauvegardé annulé sans faux fragment final visible", () => {
    const { getByRole, getByText, queryByText } = render(
      <SavedToolTimeline
        messageId="m-cancelled"
        segments={[
          { thinking: "saved thinking", content: "work note", tools: [{ name: "bash", summary: "x" }], phase: "work" },
          { content: "last partial fragment", tools: [], phase: "work" },
        ]}
        tps={0}
        totalElapsedMs={45_000}
      />,
    );

    const toggle = getByRole("button", { name: /Worked for 45 s/ });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(queryByText("last partial fragment")).toBeNull();

    fireEvent.click(toggle);
    expect(getByText("work note")).toBeTruthy();
    expect(getByText("last partial fragment")).toBeTruthy();
    expect(getByText("bash")).toBeTruthy();
  });

  it("n'affiche pas de résumé quand il n'y a pas de phase de travail", () => {
    const { getByText, queryByText } = render(
      <SavedToolTimeline
        messageId="m2"
        segments={[{ content: "plain final", tools: [] }]}
        tps={0}
        totalElapsedMs={0}
      />,
    );

    expect(getByText("plain final")).toBeTruthy();
    expect(queryByText(/Worked for|Work completed/)).toBeNull();
  });

  it("garde visible une réponse finale seule même si elle a une phase explicite", () => {
    const { getByTestId, getByText, queryByText } = render(
      <SavedToolTimeline
        messageId="m-final-only"
        segments={[{ content: "final only answer", tools: [], phase: "final" }]}
        tps={0}
        totalElapsedMs={3_000}
      />,
    );
    expect(getByText("final only answer")).toBeTruthy();
    expect(getByTestId("assistant")).toHaveAttribute("data-elapsed", "3000");
    expect(queryByText(/Worked for|Work completed/)).toBeNull();
  });
});
