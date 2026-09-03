import { UI_LIMITS, UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import type { ExtensionUiPlacementKey } from "@/types/extension-ui-contract.generated";
import type { AdvancedCleanup, AdvancedMount } from "./advanced-types";

const PLACEMENTS = new Set<string>(UI_PLACEMENTS.map(({ key }) => key));

export function advancedSlotAttributes(placement: ExtensionUiPlacementKey) {
  return { "data-extension-ui-slot": placement } as const;
}

export function createAdvancedMountManager(document: Document) {
  const cleanups: AdvancedCleanup[] = [];
  let mounts = 0;
  let explicitNoMount = false;
  let active = true;

  function mount(placement: ExtensionUiPlacementKey, render: AdvancedMount): void {
    if (!active
      || !PLACEMENTS.has(placement)
      || typeof render !== "function"
      || mounts >= UI_LIMITS.maxAdvancedMountsPerExtension) {
      throw new Error("extension_ui_mount_failed");
    }
    const selector = `[data-extension-ui-slot="${placement}"]`;
    const anchor = document.querySelector(selector);
    const view = document.defaultView;
    if (!view || !(anchor instanceof view.HTMLElement)) {
      throw new Error("extension_ui_mount_failed");
    }
    const container = document.createElement("span");
    container.dataset.extensionUiAdvancedMount = placement;
    anchor.append(container);
    try {
      const cleanup = render(container);
      if (cleanup !== undefined && typeof cleanup !== "function") {
        throw new Error("extension_ui_mount_failed");
      }
      cleanups.push(async () => {
        try { await cleanup?.(); } finally { container.remove(); }
      });
      mounts += 1;
    } catch (error) {
      container.remove();
      throw error;
    }
  }

  return {
    mount,
    completeWithoutMounts: () => {
      if (!active) throw new Error("extension_ui_mount_failed");
      explicitNoMount = true;
    },
    completed: () => mounts > 0 || explicitNoMount,
    cleanup: async () => {
      active = false;
      for (const cleanup of cleanups.reverse()) {
        try { await cleanup(); } catch { /* chaque nettoyage reste isolé */ }
      }
      cleanups.length = 0;
    },
  };
}
