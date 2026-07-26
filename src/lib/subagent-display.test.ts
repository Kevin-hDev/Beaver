import { describe, expect, it } from "vitest";
import { subagentDisplayName, subagentSecondaryText } from "./subagent-display";
import type { SubagentInfo } from "@/types/agent";

describe("subagent-display", () => {
  it("affiche toujours l'identité produit du sous-agent", () => {
    expect(subagentDisplayName(agent("coder", "Audit code"))).toBe("Claudiator");
    expect(subagentDisplayName(agent("explorer", "Audit web"))).toBe("Geminitor");
  });

  it("garde l'ancien nom libre comme texte secondaire si besoin", () => {
    expect(subagentSecondaryText(agent("explorer", "Audit web"))).toBe("Audit web");
    expect(subagentSecondaryText({
      ...agent("coder", "Audit code"),
      description: "Mission code",
    })).toBe("Mission code");
  });
});

function agent(type: "explorer" | "coder", name: string): SubagentInfo {
  return {
    sessionId: "child",
    name,
    type,
    status: "running",
    promptPreview: "",
  };
}
