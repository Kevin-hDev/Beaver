import { Fragment, type ReactNode } from "react";
import { useSlotOccupants } from "./slot-contexts";
import type { SlotOccupant, SlotPlacement } from "./slot-types";

interface SlotRendererProps<Context> {
  placement: SlotPlacement;
  context: Context;
  occupantId?: string;
  source?: "all" | SlotOccupant["source"]["kind"];
  render: (occupant: SlotOccupant, context: Context) => ReactNode;
}

export function SlotRenderer<Context>({
  placement,
  context,
  occupantId,
  source = "all",
  render,
}: SlotRendererProps<Context>) {
  const occupants = useSlotOccupants(placement);
  return occupants
    .filter((occupant) => !occupantId || occupant.id === occupantId)
    .filter((occupant) => source === "all" || occupant.source.kind === source)
    .map((occupant) => (
      <Fragment key={occupant.id}>{render(occupant, context)}</Fragment>
    ));
}
