import { extensionOccupantId, type SlotMutation } from "../slot-types";
import type { StandardCatalogSnapshot, StandardContribution } from "./types";

export function catalogMutations(snapshot: StandardCatalogSnapshot | null): SlotMutation[] {
  if (!snapshot) return [];
  return snapshot.contributions.flatMap((entry) => {
    const contribution = entry.contribution;
    if (contribution.type === "theme") return [];
    return [toMutation(entry.extensionId, contribution)];
  });
}

function toMutation(extensionId: string, contribution: StandardContribution): SlotMutation {
  const operation = contribution.operation ?? "add";
  return {
    extensionId,
    contributionId: contribution.id,
    operation,
    placement: contribution.placement,
    contributionType: contribution.type,
    order: contribution.order,
    ...(contribution.targetId ? {
      targetId: contribution.targetId.startsWith("beaver.")
        ? contribution.targetId
        : extensionOccupantId(extensionId, contribution.targetId) ?? undefined,
    } : {}),
  };
}
