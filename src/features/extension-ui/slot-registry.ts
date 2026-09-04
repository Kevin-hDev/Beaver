import { UI_LIMITS, UI_VALIDATION } from "@/types/extension-ui-contract.generated";
import type {
  SlotDefinition,
  SlotOccupant,
  SlotPlacement,
  SlotRegistry,
} from "./slot-types";

export function createSlotRegistry(
  definitions: readonly SlotDefinition[],
  coreOccupants: readonly SlotOccupant[],
): SlotRegistry {
  const definitionEntries: Array<[SlotPlacement, SlotDefinition]> = [];
  const coreEntries: Array<[SlotPlacement, SlotOccupant[]]> = [];
  const placements = new Set<SlotPlacement>();

  for (const definition of definitions) {
    if (placements.has(definition.key)) throw new Error("Duplicate slot definition.");
    placements.add(definition.key);
    definitionEntries.push([definition.key, definition]);
    coreEntries.push([definition.key, []]);
  }

  const definitionRecord = Object.fromEntries(definitionEntries) as Record<SlotPlacement, SlotDefinition>;
  const coreRecord = Object.fromEntries(coreEntries) as Record<SlotPlacement, SlotOccupant[]>;
  const occupantIds = new Set<string>();

  for (const occupant of coreOccupants) {
    const definition = definitionRecord[occupant.placement];
    if (!definition) throw new Error("Unknown slot placement.");
    if (definition.contributionType !== occupant.contributionType) {
      throw new Error("Incompatible slot occupant.");
    }
    if (occupantIds.has(occupant.id)) throw new Error("Duplicate slot occupant.");
    if (!Number.isInteger(occupant.order)
      || occupant.order < UI_VALIDATION.minOrder
      || occupant.order > UI_VALIDATION.maxOrder) {
      throw new Error("Invalid slot order.");
    }
    const placementOccupants = coreRecord[occupant.placement];
    if (placementOccupants.length >= UI_LIMITS.maxOccupantsPerPlacement) {
      throw new Error("Slot occupant limit exceeded.");
    }
    occupantIds.add(occupant.id);
    placementOccupants.push(occupant);
  }

  for (const occupants of Object.values(coreRecord)) occupants.sort(compareOccupants);
  return { definitions: definitionRecord, coreByPlacement: coreRecord };
}

export function compareOccupants(left: SlotOccupant, right: SlotOccupant): number {
  return left.order - right.order
    || sourceKey(left).localeCompare(sourceKey(right))
    || left.id.localeCompare(right.id);
}

function sourceKey(occupant: SlotOccupant): string {
  return occupant.source.kind === "core"
    ? `0:${occupant.id}`
    : `1:${occupant.source.extensionId}:${occupant.source.contributionId}`;
}
