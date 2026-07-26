import { describe, expect, it } from "vitest";
import {
  allItemIds,
  buildSourceSelection,
  createImportDraft,
  draftMatchesSource,
  selectionMode,
  toggleDraftId,
} from "@/lib/agent-import-selection";
import type { AgentSourceSummary } from "@/types/agent-import";

const source: AgentSourceSummary = {
  id: "claude",
  displayName: "Claude Code",
  status: "detected",
  partial: false,
  configured: false,
  enabled: false,
  documents: [],
  rules: [],
  skills: [
    {
      id: "one",
      name: "one",
      description: "",
      sourceId: "claude",
      sourceName: "Claude Code",
      kind: "skill",
      selected: true,
      available: true,
      updateAvailable: false,
    },
    {
      id: "two",
      name: "two",
      description: "",
      sourceId: "claude",
      sourceName: "Claude Code",
      kind: "skill",
      selected: true,
      available: true,
      updateAvailable: false,
    },
  ],
};

describe("agent-import-selection", () => {
  it("active tous les éléments proposés au premier scan", () => {
    expect([...createImportDraft(source).skillIds]).toEqual(["one", "two"]);
  });

  it("distingue les modes all, none et custom", () => {
    expect(selectionMode(2, 2)).toBe("all");
    expect(selectionMode(0, 2)).toBe("none");
    expect(selectionMode(1, 2)).toBe("custom");
  });

  it("construit une sélection personnalisée", () => {
    const draft = createImportDraft(source);
    draft.skillIds = toggleDraftId(draft.skillIds, "two");

    const selection = buildSourceSelection(source, draft);

    expect(selection.skillMode).toBe("custom");
    expect(selection.selectedSkillIds).toEqual(["one"]);
  });

  it("détecte uniquement les changements réels de sélection", () => {
    const draft = createImportDraft(source);
    expect(draftMatchesSource(source, draft)).toBe(true);

    draft.skillIds = toggleDraftId(draft.skillIds, "two");
    expect(draftMatchesSource(source, draft)).toBe(false);
  });

  it("Tout ignore un élément indisponible", () => {
    const unavailable = {
      ...source.skills[1],
      available: false,
    };
    expect([...allItemIds([source.skills[0], unavailable])]).toEqual(["one"]);
  });
});
