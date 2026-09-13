import { cleanup, render } from "@testing-library/react";
import { readFileSync } from "node:fs";
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

  it("grossit les points fins à 1,5 et garde les gros points de Claudiator", () => {
    const { container } = render(<SubagentIcon agent={agent("coder", "running")} />);
    const radii = [...container.querySelectorAll("circle")].map((circle) => circle.getAttribute("r"));

    expect(radii.filter((r) => r === "2")).toHaveLength(3);
    expect(radii.filter((r) => r === "1.5")).toHaveLength(6);
  });

  it("fait descendre la lumière : le retard de chaque point suit sa hauteur", () => {
    const { container } = render(<SubagentIcon agent={agent("explorer", "running")} />);
    const circles = [...container.querySelectorAll<SVGCircleElement>("circle")]
      .sort((a, b) => Number(a.getAttribute("cy")) - Number(b.getAttribute("cy")));
    const ranks = circles.map((circle) => Number(circle.style.getPropertyValue("--sai-rank")));

    expect(ranks[0]).toBeCloseTo(5 / 18.5, 2);
    expect(ranks[ranks.length - 1]).toBe(1);
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    expect(container.querySelector("[class*='sai-node-']")).toBeNull();
  });

  it("anime la lumière et non la forme, et s'arrête en mouvement réduit", () => {
    const css = readFileSync("src/components/agent-local/subagent-icon.css", "utf8");

    expect(css.match(/@keyframes/g)).toHaveLength(1);
    expect(css).toContain("@keyframes sai-wave");
    expect(css).not.toContain("transform");
    expect(css).toMatch(
      /@media \(prefers-reduced-motion: reduce\)\s*\{\s*\.sai-running \.sai-node\s*\{\s*animation:\s*none;/s,
    );
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
