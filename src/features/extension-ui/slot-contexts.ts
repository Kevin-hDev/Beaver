import { createContext, useContext, useMemo } from "react";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type {
  SlotContributionType,
  SlotOccupant,
  SlotPlacement,
  SlotResolution,
} from "./slot-types";
import {
  navigationAvailabilityFromResolution,
  type NavigationAvailability,
} from "./slot-navigation";

export const SlotResolutionContext = createContext<SlotResolution | null>(null);

export function useSlotOccupants(placement: SlotPlacement): readonly SlotOccupant[] {
  return useSlotResolution().occupantsByPlacement[placement];
}

export function useSlotOccupantByTarget(
  target: string,
  contributionType: SlotContributionType,
): SlotOccupant | undefined {
  const resolution = useSlotResolution();
  for (const occupants of Object.values(resolution.occupantsByPlacement)) {
    const occupant = occupants.find(
      (item) => item.target === target && item.contributionType === contributionType,
    );
    if (occupant) return occupant;
  }
  return undefined;
}

export function useNavigationAvailability(): NavigationAvailability {
  const resolution = useSlotResolution();
  return useMemo(() => navigationAvailabilityFromResolution(resolution), [resolution]);
}

export function allowsThirdPartyComposerUi(
  permissionMode: PermissionMode,
  planModeEnabled: boolean,
): boolean {
  return composerSurfaceMode(permissionMode, planModeEnabled) !== "chat";
}

export function composerSurfaceMode(
  permissionMode: PermissionMode,
  planModeEnabled: boolean,
): "chat" | "agent" | "plan" {
  if (permissionMode === "chat") return "chat";
  return planModeEnabled ? "plan" : "agent";
}

function useSlotResolution(): SlotResolution {
  const resolution = useContext(SlotResolutionContext);
  if (!resolution) throw new Error("Slot consumers must be rendered inside SlotProvider.");
  return resolution;
}
