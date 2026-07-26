import { describe, expect, it } from "vitest";
import { shouldAutoHideSidebarForAgentPanels } from "../agent-panels-auto-sidebar";

describe("shouldAutoHideSidebarForAgentPanels", () => {
  it("masque seulement quand preview et arborescence sont ouvertes et trop serrees", () => {
    expect(shouldAutoHideSidebarForAgentPanels(760, true, true)).toBe(true);
    expect(shouldAutoHideSidebarForAgentPanels(780, true, true)).toBe(false);
    expect(shouldAutoHideSidebarForAgentPanels(760, true, false)).toBe(false);
    expect(shouldAutoHideSidebarForAgentPanels(760, false, true)).toBe(false);
  });
});
