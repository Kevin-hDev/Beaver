import { useMemo, type ReactNode } from "react";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "./core-occupants";
import { SlotResolutionContext } from "./slot-contexts";
import { createSlotRegistry } from "./slot-registry";
import { resolveSlots } from "./slot-resolution";
import { useOptionalStandardCatalog } from "./standard/catalog-context";
import { catalogMutations } from "./standard/catalog-slot-adapter";

const CORE_SLOT_REGISTRY = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);

export function SlotProvider({ children }: { children: ReactNode }) {
  const snapshot = useOptionalStandardCatalog()?.snapshot ?? null;
  const mutations = useMemo(() => catalogMutations(snapshot), [snapshot]);
  const resolution = useMemo(
    () => resolveSlots(CORE_SLOT_REGISTRY, mutations),
    [mutations],
  );
  return (
    <SlotResolutionContext.Provider value={resolution}>
      {children}
    </SlotResolutionContext.Provider>
  );
}
