import { describe, expect, it } from "vitest";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "../core-occupants";
import { createSlotRegistry } from "../slot-registry";
import type { SlotOccupant } from "../slot-types";

describe("slot registry", () => {
  it("contains each Beaver occupant exactly once", () => {
    const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);
    const ids = Object.values(registry.coreByPlacement).flat().map((occupant) => occupant.id);

    expect(ids).toHaveLength(new Set(ids).size);
    expect(ids).toContain("beaver.settings");
    expect(ids).toContain("beaver.extensions");
    expect(ids).toContain("beaver.composer-menu");
  });

  it("projects all eight generated placements without an extra authority", () => {
    const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);

    expect(Object.keys(registry.definitions)).toEqual(UI_PLACEMENTS.map(({ key }) => key));
    expect(registry.coreByPlacement["app.navigation.primary"]).toHaveLength(4);
    expect(registry.coreByPlacement["settings.navigation.preferences"]).toHaveLength(3);
    expect(registry.coreByPlacement["settings.navigation.agent"]).toHaveLength(4);
    expect(registry.coreByPlacement["settings.navigation.models"]).toHaveLength(3);
    expect(registry.coreByPlacement["settings.navigation.integrations"]).toHaveLength(4);
    expect(registry.coreByPlacement["settings.navigation.application"]).toHaveLength(3);
  });

  it("refuses duplicate, unknown or type-incompatible core declarations", () => {
    const first = CORE_SLOT_OCCUPANTS[0];

    expect(() => createSlotRegistry(UI_PLACEMENTS, [first, first])).toThrow();
    expect(() => createSlotRegistry(UI_PLACEMENTS, [{
      ...first,
      placement: "missing.placement",
    } as unknown as SlotOccupant])).toThrow();
    expect(() => createSlotRegistry(UI_PLACEMENTS, [{
      ...first,
      contributionType: "action",
    }])).toThrow();
  });
});
