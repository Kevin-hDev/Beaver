import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SubagentIcon } from "../subagent-icon";
import type { SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());

describe("SubagentIcon", () => {
  it("utilise l'icône Claudiator animée quand le coder tourne", () => {
    const { container } = render(<SubagentIcon agent={agent("coder", "running")} />);
    const icon = container.querySelector(".sai-icon");

    expect(icon).toHaveClass("sai-claudiator");
    expect(icon).toHaveClass("sai-running");
    expect(container.querySelectorAll("circle")).toHaveLength(9);
  });

  it("utilise l'icône Geminitor figée quand l'explorer est terminé", () => {
    const { container } = render(<SubagentIcon agent={agent("explorer", "completed")} />);
    const icon = container.querySelector(".sai-icon");

    expect(icon).toHaveClass("sai-geminitor");
    expect(icon).not.toHaveClass("sai-running");
    expect(container.querySelectorAll("circle")).toHaveLength(10);
  });
});

function agent(type: "explorer" | "coder", status: SubagentInfo["status"]): SubagentInfo {
  return {
    sessionId: `${type}-child`,
    name: type,
    type,
    status,
    promptPreview: "",
  };
}
