import { EXTENSION_UI_API_VERSION } from "@/types/extension-ui-contract.generated";
import type { AdvancedExtensionContext } from "./advanced-types";

export function createAdvancedContext(
  extensionId: string,
  mounts: Pick<AdvancedExtensionContext, "mount" | "completeWithoutMounts">,
): AdvancedExtensionContext {
  return Object.freeze({
    apiVersion: EXTENSION_UI_API_VERSION,
    extensionId,
    mount: mounts.mount,
    completeWithoutMounts: mounts.completeWithoutMounts,
  });
}
