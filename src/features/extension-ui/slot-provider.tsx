import { useMemo, type ReactNode } from "react";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "./core-occupants";
import { SlotResolutionContext } from "./slot-contexts";
import { createSlotRegistry } from "./slot-registry";
import { resolveSlots } from "./slot-resolution";

const CORE_SLOT_REGISTRY = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);

export function SlotProvider({ children }: { children: ReactNode }) {
  /* UI-P1 migre seulement le cœur : l'absence de mutations tierces ici est une
     frontière volontaire, levée uniquement lorsque le catalogue validé existe. */
  const resolution = useMemo(() => resolveSlots(CORE_SLOT_REGISTRY, []), []);
  return (
    <SlotResolutionContext.Provider value={resolution}>
      {children}
    </SlotResolutionContext.Provider>
  );
}
