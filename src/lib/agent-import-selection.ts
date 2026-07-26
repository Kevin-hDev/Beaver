import type {
  AgentImportItem,
  AgentSelectionMode,
  AgentSourceSelection,
  AgentSourceSummary,
} from "@/types/agent-import";

export interface AgentImportDraft {
  skillIds: Set<string>;
  ruleIds: Set<string>;
  documentIds: Set<string>;
}

export function createImportDraft(source: AgentSourceSummary): AgentImportDraft {
  return {
    skillIds: selectedIds(source.skills),
    ruleIds: selectedIds(source.rules),
    documentIds: selectedIds(source.documents),
  };
}

export function selectionMode(
  selectedCount: number,
  totalCount: number,
): AgentSelectionMode {
  if (selectedCount === 0) return "none";
  if (selectedCount === totalCount) return "all";
  return "custom";
}

export function buildSourceSelection(
  source: AgentSourceSummary,
  draft: AgentImportDraft,
  enabled = true,
): AgentSourceSelection {
  return {
    sourceId: source.id,
    enabled,
    skillMode: selectionMode(draft.skillIds.size, source.skills.length),
    selectedSkillIds: [...draft.skillIds],
    selectedRuleIds: [...draft.ruleIds],
    selectedDocumentIds: [...draft.documentIds],
  };
}

export function draftMatchesSource(
  source: AgentSourceSummary,
  draft: AgentImportDraft,
): boolean {
  const saved = createImportDraft(source);
  return sameIds(draft.skillIds, saved.skillIds)
    && sameIds(draft.ruleIds, saved.ruleIds)
    && sameIds(draft.documentIds, saved.documentIds);
}

export function toggleDraftId(current: Set<string>, id: string): Set<string> {
  const next = new Set(current);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

export function allItemIds(items: AgentImportItem[]): Set<string> {
  return new Set(items.filter((item) => item.available).map((item) => item.id));
}

function selectedIds(items: AgentImportItem[]): Set<string> {
  return new Set(
    items
      .filter((item) => item.selected && item.available)
      .map((item) => item.id),
  );
}

function sameIds(left: Set<string>, right: Set<string>): boolean {
  return left.size === right.size && [...left].every((id) => right.has(id));
}
