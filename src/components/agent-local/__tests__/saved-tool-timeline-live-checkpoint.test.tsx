import type { ComponentProps, ComponentType } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { SavedToolTimeline } from "../message-tool-timeline";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretRight: () => <span />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key === "agentLocal.workSummaryNoDuration" ? "Work completed" : key,
  }),
}));
vi.mock("../assistant-message", () => ({
  AssistantMessage: ({ content }: { content: string }) => <div>{content}</div>,
}));
vi.mock("../thinking-section", () => ({ ThinkingSection: () => <div /> }));
vi.mock("../tool-bubble", () => ({
  ToolBubble: () => <div />,
  SavedToolBubble: () => <div />,
}));
vi.mock("../branch-bubble", () => ({ BranchBubble: () => <div /> }));

type LiveCheckpointProps = ComponentProps<typeof SavedToolTimeline> & {
  liveCheckpoint?: boolean;
};
const LiveCheckpointTimeline = SavedToolTimeline as ComponentType<LiveCheckpointProps>;

describe("saved live checkpoint timeline", () => {
  it("reste ouverte tant que le stream parent continue", () => {
    const view = render(
      <LiveCheckpointTimeline
        messageId="checkpoint"
        segments={[{
          content: "Travail visible", tools: [], phase: "work",
        }]}
        tps={0}
        totalElapsedMs={0}
        liveCheckpoint
      />,
    );

    expect(view.queryByRole("button", { name: "Work completed" })).toBeNull();
    expect(view.getByText("Travail visible")).toBeTruthy();
  });
});
