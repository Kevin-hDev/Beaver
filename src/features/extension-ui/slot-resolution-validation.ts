import { UI_PROTECTED_OCCUPANTS, UI_VALIDATION } from "@/types/extension-ui-contract.generated";
import { extensionOccupantId, type SlotDiagnostic, type SlotMutation, type SlotOccupant } from "./slot-types";

const DESTRUCTIVE = new Set<SlotMutation["operation"]>(["replace", "move", "remove"]);

export type RejectMutation = (
  mutation: SlotMutation,
  code: SlotDiagnostic["code"],
) => void;

export function mutationId(mutation: SlotMutation): string {
  return extensionOccupantId(mutation.extensionId, mutation.contributionId)
    ?? `invalid:${mutation.extensionId.length}:${mutation.contributionId.length}`;
}

export function compareMutations(left: SlotMutation, right: SlotMutation): number {
  return mutationId(left).localeCompare(mutationId(right));
}

export function compareDiagnostics(left: SlotDiagnostic, right: SlotDiagnostic): number {
  return left.contributionId.localeCompare(right.contributionId) || left.code.localeCompare(right.code);
}

export function validOrder(order: number): boolean {
  return Number.isInteger(order)
    && order >= UI_VALIDATION.minOrder
    && order <= UI_VALIDATION.maxOrder;
}

export function needsNewOccupant(operation: SlotMutation["operation"]): boolean {
  return operation === "add" || operation === "before"
    || operation === "after" || operation === "replace";
}

export function isProtected(target: SlotOccupant, operation: SlotMutation["operation"]): boolean {
  return UI_PROTECTED_OCCUPANTS.some((rule) => rule.placement === target.placement
    && rule.occupant === target.id
    && rule.operations.some((protectedOperation) => protectedOperation === operation));
}

export function targetDiagnostic(
  mutation: SlotMutation,
  references: ReadonlyMap<string, SlotOccupant>,
): SlotDiagnostic["code"] | null {
  if (mutation.operation === "add") return null;
  const target = mutation.targetId ? references.get(mutation.targetId) : undefined;
  if (!target) return "ui_reference_missing";
  const compatible = mutation.operation === "move"
    ? target.contributionType === mutation.contributionType
    : target.placement === mutation.placement
      && target.contributionType === mutation.contributionType;
  if (!compatible) return "ui_reference_incompatible";
  return isProtected(target, mutation.operation) ? "ui_protected_occupant" : null;
}

export function rejectUnavailableDependencies(
  mutations: readonly SlotMutation[],
  declaredCandidateIds: ReadonlySet<string>,
  rejected: ReadonlySet<string>,
  reject: RejectMutation,
): void {
  let changed = true;
  while (changed) {
    changed = false;
    for (const mutation of mutations) {
      const id = mutationId(mutation);
      if (rejected.has(id) || !mutation.targetId
        || !declaredCandidateIds.has(mutation.targetId)
        || !rejected.has(mutation.targetId)) continue;
      reject(mutation, "ui_reference_missing");
      changed = true;
    }
  }
}

export function rejectPreflightConflicts(
  mutations: readonly SlotMutation[],
  reject: RejectMutation,
): void {
  rejectGroups(groupMutations(mutations, mutationId), reject);
  const destructive = mutations.filter(
    ({ operation, targetId }) => DESTRUCTIVE.has(operation) && targetId,
  );
  rejectGroups(groupMutations(destructive, ({ targetId }) => targetId ?? ""), reject);
}

function groupMutations(
  mutations: readonly SlotMutation[],
  keyOf: (mutation: SlotMutation) => string,
): Map<string, SlotMutation[]> {
  const groups = new Map<string, SlotMutation[]>();
  for (const mutation of mutations) {
    const key = keyOf(mutation);
    const group = groups.get(key) ?? [];
    group.push(mutation);
    groups.set(key, group);
  }
  return groups;
}

function rejectGroups(groups: ReadonlyMap<string, SlotMutation[]>, reject: RejectMutation): void {
  for (const group of groups.values()) {
    if (group.length > 1) group.forEach((mutation) => reject(mutation, "ui_mutation_conflict"));
  }
}
