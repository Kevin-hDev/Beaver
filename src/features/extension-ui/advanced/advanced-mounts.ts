import { UI_LIMITS, UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import type { ExtensionUiPlacementKey } from "@/types/extension-ui-contract.generated";
import type { AdvancedCleanup, AdvancedMount } from "./advanced-types";

import { runAdvancedCleanups } from "./advanced-cleanup";

const PLACEMENTS = new Set<string>(UI_PLACEMENTS.map(({ key }) => key));

interface MountRequest {
  placement: ExtensionUiPlacementKey;
  render: AdvancedMount;
  container: HTMLElement | null;
  cleanup: AdvancedCleanup | null;
  failed: boolean;
}

export function advancedSlotAttributes(placement: ExtensionUiPlacementKey) {
  return { "data-extension-ui-slot": placement } as const;
}

export function createAdvancedMountManager(document: Document) {
  const requests: MountRequest[] = [];
  let explicitNoMount = false;
  let active = true;
  let reconciling = false;
  let reconcileRequested = false;
  const observer = new MutationObserver(() => { void reconcile(); });
  observer.observe(document.documentElement, { childList: true, subtree: true });

  function mount(placement: ExtensionUiPlacementKey, render: AdvancedMount): void {
    if (!active || !PLACEMENTS.has(placement) || typeof render !== "function"
      || requests.length >= UI_LIMITS.maxAdvancedMountsPerExtension) {
      throw new Error("extension_ui_mount_failed");
    }
    const request: MountRequest = {
      placement, render, container: null, cleanup: null, failed: false,
    };
    requests.push(request);
    mountAvailable(request, true);
  }

  async function reconcile(
    deadline = performance.now() + UI_LIMITS.maxAdvancedCleanupMs,
  ): Promise<void> {
    if (!active) return;
    if (reconciling) {
      reconcileRequested = true;
      return;
    }
    reconciling = true;
    reconcileRequested = false;
    try {
      const callbacks = requests.flatMap((request) =>
        request.container && !request.container.isConnected ? detachRequest(request) : []);
      await runAdvancedCleanups(callbacks, deadline);
      // The observer may have been disconnected while a cleanup callback was pending.
      if (!active) return;
      for (const request of requests) {
        if (!request.container && !request.failed) mountAvailable(request, false);
      }
    } finally {
      reconciling = false;
      // Mutations delivered during the await still need a pass, within the same budget.
      if (active && reconcileRequested) void reconcile(deadline);
    }
  }

  function mountAvailable(request: MountRequest, propagate: boolean): void {
    if (!active) return;
    const selector = `[data-extension-ui-slot="${request.placement}"]`;
    const anchor = document.querySelector(selector);
    const view = document.defaultView;
    if (!view || !(anchor instanceof view.HTMLElement)) return;
    const container = document.createElement("span");
    container.dataset.extensionUiAdvancedMount = request.placement;
    anchor.append(container);
    try {
      const cleanup = request.render(container);
      if (cleanup !== undefined && typeof cleanup !== "function") {
        throw new Error("extension_ui_mount_failed");
      }
      request.container = container;
      request.cleanup = cleanup ?? null;
    } catch (error) {
      container.remove();
      request.failed = true;
      if (propagate) throw error;
    }
  }

  function detachRequest(request: MountRequest): AdvancedCleanup[] {
    const cleanup = request.cleanup;
    request.cleanup = null;
    request.container?.remove();
    request.container = null;
    return cleanup ? [cleanup] : [];
  }

  return {
    mount,
    completeWithoutMounts: () => {
      if (!active) throw new Error("extension_ui_mount_failed");
      explicitNoMount = true;
    },
    completed: () => requests.length > 0 || explicitNoMount,
    detach: () => {
      active = false;
      observer.disconnect();
      return requests.splice(0).reverse().flatMap(detachRequest);
    },
  };
}
