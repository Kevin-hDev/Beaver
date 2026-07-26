import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AgentSourceLogo } from "../agent-source-logo";

const sources = [
  ["claude", "Claude Code"],
  ["codex", "Codex"],
  ["agents", "Agents"],
  ["hermes", "Hermes Agent"],
  ["qwen", "Qwen Code"],
  ["zcode", "ZCode"],
  ["openclaw", "OpenClaw"],
  ["opencode", "OpenCode"],
  ["kimi", "Kimi Code"],
] as const;

describe("AgentSourceLogo", () => {
  it.each(sources)("affiche le logo de détail de %s", (sourceId, displayName) => {
    const { container } = render(
      <AgentSourceLogo
        sourceId={sourceId}
        displayName={displayName}
        variant="detail"
      />,
    );

    expect(screen.getByRole("img", { name: displayName })).toBeInTheDocument();
    expect(container.querySelector("svg")).not.toBeInTheDocument();
  });

  it("utilise les portraits Hermes adaptés aux deux thèmes sur les cartes", () => {
    const { container } = render(
      <AgentSourceLogo
        sourceId="hermes"
        displayName="Hermes Agent"
        variant="card"
      />,
    );

    expect(container.querySelector(".themed-icon-light")).toBeInTheDocument();
    expect(container.querySelector(".themed-icon-dark")).toBeInTheDocument();
  });

  it("conserve les proportions lisibles du logo Moonshot AI", () => {
    render(
      <AgentSourceLogo
        sourceId="kimi"
        displayName="Kimi Code"
        variant="detail"
      />,
    );

    expect(screen.getByRole("img", { name: "Kimi Code" })).toHaveStyle({
      "--aim-logo-ratio": "6.02",
    });
  });
});
