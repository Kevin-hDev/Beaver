import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import { orderOccupants, type RelativePlacement } from "./slot-resolution-order";
import type {
  SlotMutation,
  SlotOccupant,
  SlotPlacement,
  SlotRegistry,
} from "./slot-types";
import {
  mutationId,
  type RejectMutation,
} from "./slot-resolution-validation";

export function applyMutation(
  mutation: SlotMutation,
  candidate: SlotOccupant | undefined,
  working: Record<SlotPlacement, SlotOccupant[]>,
  relations: RelativePlacement[],
  reject: RejectMutation,
): void {
  if (mutation.operation === "add" && candidate) return;
  const target = mutation.targetId ? occupantIndex(working).get(mutation.targetId) : undefined;
  if (!target) {
    reject(mutation, "ui_reference_missing");
    return;
  }
  if ((mutation.operation === "before" || mutation.operation === "after") && candidate) {
    relations.push({ occupantId: candidate.id, operation: mutation.operation, targetId: target.id });
  } else if (mutation.operation === "replace" && candidate) {
    removeOccupant(working, target.id);
    updateOccupant(working, candidate.id, { order: target.order });
  } else if (mutation.operation === "move") {
    removeOccupant(working, target.id);
    working[mutation.placement].push({
      ...target,
      placement: mutation.placement,
      order: mutation.order,
    });
  } else if (mutation.operation === "remove") {
    removeOccupant(working, target.id);
  }
}

export function materializeCandidates(
  working: Record<SlotPlacement, SlotOccupant[]>,
  candidates: Iterable<SlotOccupant>,
): void {
  for (const candidate of candidates) working[candidate.placement].push(candidate);
}

export function rejectBrokenRelations(
  relations: readonly RelativePlacement[],
  working: Record<SlotPlacement, SlotOccupant[]>,
  mutations: readonly SlotMutation[],
  rejected: ReadonlySet<string>,
  reject: RejectMutation,
): void {
  const ids = occupantIndex(working);
  for (const relation of relations) {
    if (rejected.has(relation.occupantId)) continue;
    const mutation = mutations.find((item) => mutationId(item) === relation.occupantId);
    if (!mutation) continue;
    const target = ids.get(relation.targetId);
    if (!target) {
      reject(mutation, "ui_reference_missing");
      continue;
    }
    // A valid preflight target may move; relative constraints must still share
    // the resolved placement and contribution type before ordering.
    if (
      target.placement !== mutation.placement
      || target.contributionType !== mutation.contributionType
    ) {
      reject(mutation, "ui_reference_incompatible");
    }
  }
}

export function orderAndBound(
  working: Record<SlotPlacement, SlotOccupant[]>,
  relations: readonly RelativePlacement[],
  mutations: readonly SlotMutation[],
  rejected: Set<string>,
  reject: RejectMutation,
): Record<SlotPlacement, readonly SlotOccupant[]> {
  for (const placement of Object.keys(working) as SlotPlacement[]) {
    const placementRelations = relations.filter(
      (relation) => working[placement].some(({ id }) => id === relation.occupantId),
    );
    let result = orderOccupants(working[placement], placementRelations);
    if (result.cyclicIds.length > 0) {
      for (const id of result.cyclicIds) {
        const mutation = mutations.find((item) => mutationId(item) === id);
        if (mutation) reject(mutation, "ui_mutation_conflict");
      }
      removeRejected(working, rejected);
      const survivingIds = new Set(working[placement].map(({ id }) => id));
      const survivingRelations = placementRelations.filter(
        ({ occupantId, targetId }) => survivingIds.has(occupantId) && survivingIds.has(targetId),
      );
      result = orderOccupants(working[placement], survivingRelations);
    }
    const ordered = result.ordered;
    while (ordered.length > UI_LIMITS.maxOccupantsPerPlacement) {
      const extensionIndex = lastExtensionIndex(ordered);
      if (extensionIndex < 0) break;
      const [removed] = ordered.splice(extensionIndex, 1);
      const mutation = mutations.find((item) => mutationId(item) === removed.id);
      if (mutation) reject(mutation, "ui_limit_exceeded");
    }
    working[placement] = ordered;
  }
  removeRejected(working, rejected);
  return working;
}

export function cloneCore(registry: SlotRegistry): Record<SlotPlacement, SlotOccupant[]> {
  return Object.fromEntries(
    Object.entries(registry.coreByPlacement).map(([placement, occupants]) => [placement, [...occupants]]),
  ) as Record<SlotPlacement, SlotOccupant[]>;
}

export function occupantIndex(
  working: Record<SlotPlacement, SlotOccupant[]>,
): Map<string, SlotOccupant> {
  return new Map(Object.values(working).flat().map((occupant) => [occupant.id, occupant]));
}

export function removeRejected(
  working: Record<SlotPlacement, SlotOccupant[]>,
  rejected: ReadonlySet<string>,
): void {
  for (const placement of Object.keys(working) as SlotPlacement[]) {
    working[placement] = working[placement].filter((occupant) => !rejected.has(occupant.id));
  }
}

function lastExtensionIndex(occupants: readonly SlotOccupant[]): number {
  for (let index = occupants.length - 1; index >= 0; index -= 1) {
    if (occupants[index].source.kind === "extension") return index;
  }
  return -1;
}

function removeOccupant(working: Record<SlotPlacement, SlotOccupant[]>, id: string): void {
  for (const placement of Object.keys(working) as SlotPlacement[]) {
    working[placement] = working[placement].filter((occupant) => occupant.id !== id);
  }
}

function updateOccupant(
  working: Record<SlotPlacement, SlotOccupant[]>,
  id: string,
  patch: Pick<SlotOccupant, "order">,
): void {
  for (const placement of Object.keys(working) as SlotPlacement[]) {
    working[placement] = working[placement]
      .map((occupant) => occupant.id === id ? { ...occupant, ...patch } : occupant);
  }
}
