/* @vitest-environment jsdom */
import { renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { navItemFromOccupant } from "@/components/layout/nav-items";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "@/features/extension-ui/core-occupants";
import { SlotResolutionContext } from "@/features/extension-ui/slot-contexts";
import { createSlotRegistry } from "@/features/extension-ui/slot-registry";
import { resolveSlots } from "@/features/extension-ui/slot-resolution";
import type { SlotOccupant } from "@/features/extension-ui/slot-types";
import { useResolvedSettingsSections } from "../settings-sections";

const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);

describe("navigation projection guards", () => {
  it("returns null for an occupant that cannot become a core navigation item", () => {
    const malformed: SlotOccupant = {
      id: "extension:com.example.ui:com.example.ui.tab",
      placement: "app.navigation.primary",
      contributionType: "tab",
      order: 1,
      source: {
        kind: "extension",
        extensionId: "com.example.ui",
        contributionId: "com.example.ui.tab",
      },
      target: "com.example.ui.tab",
    };

    expect(navItemFromOccupant(malformed)).toBeNull();
  });

  it("drops a malformed settings occupant instead of throwing", () => {
    const base = resolveSlots(registry, []);
    const valid = CORE_SLOT_OCCUPANTS.find(({ id }) => id === "beaver.general");
    if (!valid) throw new Error("Missing core settings fixture.");
    const { iconKey: _iconKey, labelKey: _labelKey, ...malformed } = valid;
    const resolution = {
      ...base,
      occupantsByPlacement: {
        ...base.occupantsByPlacement,
        "settings.navigation.preferences": [malformed],
      },
    };
    const wrapper = ({ children }: { children: ReactNode }) => (
      <SlotResolutionContext.Provider value={resolution}>
        {children}
      </SlotResolutionContext.Provider>
    );

    const { result } = renderHook(() => useResolvedSettingsSections(), { wrapper });

    expect(result.current.map(({ i18n }) => i18n)).toEqual([
      "settings.sections.agent",
      "settings.sections.models",
      "settings.sections.integrations",
      "settings.sections.application",
    ]);
  });
});
