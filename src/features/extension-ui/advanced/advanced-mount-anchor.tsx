import type { ExtensionUiPlacementKey } from "@/types/extension-ui-contract.generated";
import { advancedSlotAttributes } from "./advanced-mounts";
import "./advanced-mounts.css";

export function AdvancedMountAnchor({ placement }: { placement: ExtensionUiPlacementKey }) {
  return <span className="extension-ui-advanced-slot" {...advancedSlotAttributes(placement)} />;
}
