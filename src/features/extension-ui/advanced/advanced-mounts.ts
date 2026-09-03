import { UI_LIMITS, UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import type { ExtensionUiPlacementKey } from "@/types/extension-ui-contract.generated";
import type { AdvancedCleanup, AdvancedMount } from "./advanced-types";

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

  async function reconcile(): Promise<void> {
    if (!active || reconciling) return;
    reconciling = true;
    try {
      for (const request of requests) {
        if (request.container && !request.container.isConnected) await runCleanup(request);
        if (!request.container && !request.failed) mountAvailable(request, false);
      }
    } finally {
      reconciling = false;
    }
  }

  function mountAvailable(request: MountRequest, propagate: boolean): void {
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

  async function runCleanup(request: MountRequest): Promise<void> {
    const cleanup = request.cleanup;
    request.cleanup = null;
    request.container?.remove();
    request.container = null;
    try { await cleanup?.(); } catch { /* chaque montage reste isolé */ }
  }

  return {
    mount,
    completeWithoutMounts: () => {
      if (!active) throw new Error("extension_ui_mount_failed");
      explicitNoMount = true;
    },
    completed: () => requests.length > 0 || explicitNoMount,
    cleanup: async () => {
      active = false;
      observer.disconnect();
      for (const request of requests.reverse()) await runCleanup(request);
      requests.length = 0;
    },
  };
}
