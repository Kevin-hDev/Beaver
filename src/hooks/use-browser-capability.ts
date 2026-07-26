import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { IS_LINUX } from "@/lib/platform";

export type BrowserCapability =
  | { status: "hidden" }
  | { status: "unavailable" }
  | { status: "ready"; engineVersion: string };

const HIDDEN_CAPABILITY: BrowserCapability = { status: "hidden" };
const UNAVAILABLE_CAPABILITY: BrowserCapability = { status: "unavailable" };

export function initialBrowserCapability(isLinux: boolean): BrowserCapability {
  return isLinux ? HIDDEN_CAPABILITY : UNAVAILABLE_CAPABILITY;
}

function normalizeCapability(value: unknown): BrowserCapability {
  if (!value || typeof value !== "object") return UNAVAILABLE_CAPABILITY;
  const raw = value as Record<string, unknown>;
  if (raw.status === "hidden") return HIDDEN_CAPABILITY;
  if (raw.status === "unavailable") return UNAVAILABLE_CAPABILITY;
  if (
    raw.status === "ready" &&
    typeof raw.engineVersion === "string" &&
    raw.engineVersion.length > 0 &&
    raw.engineVersion.length <= 64
  ) {
    return { status: "ready", engineVersion: raw.engineVersion };
  }
  return UNAVAILABLE_CAPABILITY;
}

export function useBrowserCapability(): BrowserCapability {
  const [capability, setCapability] = useState<BrowserCapability>(() =>
    initialBrowserCapability(IS_LINUX));

  useEffect(() => {
    let cancelled = false;
    let eventSequence = 0;
    const unlisten = listen<unknown>("browser-capability-v1", (event) => {
      eventSequence += 1;
      if (!cancelled) setCapability(normalizeCapability(event.payload));
    });
    const initialSequence = eventSequence;
    void invoke<unknown>("browser_capability")
      .then((value) => {
        if (!cancelled && initialSequence === eventSequence) {
          setCapability(normalizeCapability(value));
        }
      })
      .catch(() => {
        if (!cancelled && initialSequence === eventSequence) {
          setCapability(UNAVAILABLE_CAPABILITY);
        }
      });
    return () => {
      cancelled = true;
      cleanupTauriListener(unlisten);
    };
  }, []);

  return capability;
}
