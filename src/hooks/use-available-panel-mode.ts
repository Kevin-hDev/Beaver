import type { PanelMode } from "./use-forecast-panel";
import { useBrowserCapability, type BrowserCapability } from "./use-browser-capability";

export function resolveAvailablePanelMode(
  requested: PanelMode,
  capability: BrowserCapability,
): PanelMode {
  if (requested === "browser" && capability.status !== "ready") return "preview";
  return requested;
}

export function useAvailablePanelMode(requested: PanelMode) {
  const capability = useBrowserCapability();
  return {
    browserStatus: capability.status,
    panelMode: resolveAvailablePanelMode(requested, capability),
  };
}
