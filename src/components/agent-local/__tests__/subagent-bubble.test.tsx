import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SubagentBubble } from "../subagent-bubble";
import type { SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const count = typeof opts?.count === "number" || typeof opts?.count === "string"
        ? opts.count
        : 0;
      if (key === "subagents.bubbleLabel") return `${count} agents créés`;
      return key;
    },
  }),
}));

describe("SubagentBubble", () => {
  it("affiche les identités produit et les couleurs dédiées", () => {
    const { container, getByText } = render(
      <SubagentBubble
        subagents={[
          agent("coder", "Audit code", "claudiator"),
          agent("explorer", "Audit web", "geminitor"),
        ]}
        onOpen={vi.fn()}
      />,
    );

    expect(container.querySelector(".sb-root")).toHaveClass("chat-column-surface");
    expect(getByText("Claudiator")).toBeTruthy();
    expect(getByText("Geminitor")).toBeTruthy();
    expect(container.querySelector(".sai-claudiator")).toBeTruthy();
    expect(container.querySelector(".sai-geminitor")).toBeTruthy();
    expect(container.querySelector(".sai-running")).toBeNull();
  });
});

function agent(
  type: "explorer" | "coder",
  description: string,
  colorKey: string,
): SubagentInfo {
  return {
    sessionId: `${type}-child`,
    name: type,
    type,
    status: "completed",
    promptPreview: "",
    description,
    colorKey,
  };
}
