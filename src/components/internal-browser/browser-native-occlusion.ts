const MAX_NATIVE_OCCLUSIONS = 16;

export const BROWSER_NATIVE_OCCLUSION_EVENT = "clgo-browser-native-occlusion-v1";

const blockers = new Set<string>();

function createToken(): string | null {
  if (typeof window === "undefined" || !window.crypto?.getRandomValues) return null;
  const bytes = new Uint8Array(16);
  window.crypto.getRandomValues(bytes);
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

function publishChange() {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new Event(BROWSER_NATIVE_OCCLUSION_EVENT));
}

function publishRevealAfterLayout() {
  if (typeof window === "undefined") return;
  window.requestAnimationFrame(publishChange);
}

export function isBrowserNativeOccluded(): boolean {
  return blockers.size > 0;
}

export function acquireBrowserNativeOcclusion(): (() => void) | null {
  if (blockers.size >= MAX_NATIVE_OCCLUSIONS) return null;
  const token = createToken();
  if (!token || blockers.has(token)) return null;
  blockers.add(token);
  publishChange();

  let released = false;
  return () => {
    if (released) return;
    released = true;
    blockers.delete(token);
    publishRevealAfterLayout();
  };
}
