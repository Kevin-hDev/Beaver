import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  BROWSER_ENGINE_STOPPED_EVENT,
  BROWSER_READY_EVENT,
  parseBrowserTabEvent,
} from "./browser-events";
import {
  BROWSER_NATIVE_OCCLUSION_EVENT,
  isBrowserNativeOccluded,
} from "./browser-native-occlusion";
import { measureBrowserSurface, sameBrowserBounds } from "./browser-surface-measure";
import { attachBrowserTransitionGuards } from "./browser-surface-transitions";
import type { BrowserSurfaceBounds, BrowserSurfaceRequest } from "./browser-types";

interface BrowserSurfaceTarget {
  conversationId: string;
  tabId: string;
  url: string | null;
}

interface BrowserSurfaceArgs extends BrowserSurfaceTarget {
  active: boolean;
  onError: () => void;
}

function sameTarget(left: BrowserSurfaceTarget | null, right: BrowserSurfaceTarget) {
  return left?.conversationId === right.conversationId &&
    left.tabId === right.tabId && left.url === right.url;
}

export function useBrowserSurface(args: BrowserSurfaceArgs) {
  const hostRef = useRef<HTMLDivElement>(null);
  const argsRef = useRef(args);
  const frameRef = useRef<number | null>(null);
  const generationRef = useRef(0);
  const lastBoundsRef = useRef<BrowserSurfaceBounds | null>(null);
  const lastTargetRef = useRef<BrowserSurfaceTarget | null>(null);
  const stoppedTargetRef = useRef<BrowserSurfaceTarget | null>(null);
  const occludedRef = useRef(isBrowserNativeOccluded());
  const movingRef = useRef<() => boolean>(() => false);
  const nativeEventRef = useRef({ conversationId: args.conversationId, generation: 0 });

  const send = useCallback((request: BrowserSurfaceRequest) => {
    void invoke("browser_surface", { request }).catch(() => {
      if (request.url) {
        stoppedTargetRef.current = {
          conversationId: request.conversationId,
          tabId: request.tabId,
          url: request.url,
        };
      }
      argsRef.current.onError();
    });
  }, []);

  const hide = useCallback(() => {
    const bounds = lastBoundsRef.current;
    const target = argsRef.current;
    if (!bounds?.visible || !lastTargetRef.current) return;
    generationRef.current += 1;
    const hidden = { ...bounds, visible: false, generation: generationRef.current };
    lastBoundsRef.current = hidden;
    lastTargetRef.current = { ...target, url: null };
    send({ ...target, url: null, bounds: hidden });
  }, [send]);

  const synchronize = useCallback(() => {
    frameRef.current = null;
    const host = hostRef.current;
    const target = argsRef.current;
    if (
      !host || !target.active || !target.url || occludedRef.current || movingRef.current()
    ) {
      hide();
      return;
    }
    const bounds = measureBrowserSurface(host, generationRef.current + 1);
    if (!bounds) return;
    const nextTarget = {
      conversationId: target.conversationId,
      tabId: target.tabId,
      url: target.url,
    };
    if (sameTarget(stoppedTargetRef.current, nextTarget)) return;
    stoppedTargetRef.current = null;
    if (sameBrowserBounds(lastBoundsRef.current, bounds) && sameTarget(lastTargetRef.current, nextTarget)) {
      return;
    }
    generationRef.current = bounds.generation;
    lastBoundsRef.current = bounds;
    lastTargetRef.current = nextTarget;
    send({ ...nextTarget, bounds });
  }, [hide, send]);

  const schedule = useCallback(() => {
    if (frameRef.current !== null) return;
    frameRef.current = window.requestAnimationFrame(synchronize);
  }, [synchronize]);

  const retrySurface = useCallback(() => {
    const target = argsRef.current;
    if (!target.active || !target.url) return;
    stoppedTargetRef.current = null;
    lastTargetRef.current = null;
    schedule();
  }, [schedule]);

  useEffect(() => {
    argsRef.current = args;
    if (nativeEventRef.current.conversationId !== args.conversationId) {
      nativeEventRef.current = { conversationId: args.conversationId, generation: 0 };
    }
  }, [args]);

  useEffect(() => {
    if (args.active && args.url) schedule();
    else hide();
  }, [args.active, args.conversationId, args.tabId, args.url, hide, schedule]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    const observer = new ResizeObserver(schedule);
    observer.observe(host);
    window.addEventListener("resize", schedule);
    const guards = attachBrowserTransitionGuards(host, hide, schedule);
    movingRef.current = guards.moving;
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", schedule);
      guards.detach();
      movingRef.current = () => false;
    };
  }, [hide, schedule]);

  useEffect(() => {
    const handleNativeEvent = (value: unknown, stopped: boolean) => {
      const target = argsRef.current;
      const event = parseBrowserTabEvent(value, target.conversationId);
      const tracked = nativeEventRef.current;
      if (
        !event || event.tabId !== target.tabId ||
        tracked.conversationId !== target.conversationId || event.generation <= tracked.generation
      ) return;
      tracked.generation = event.generation;
      if (stopped && target.url) {
        stoppedTargetRef.current = {
          conversationId: target.conversationId,
          tabId: target.tabId,
          url: target.url,
        };
        return;
      }
      schedule();
    };
    const unlistenReady = listen<unknown>(BROWSER_READY_EVENT, (event) => {
      handleNativeEvent(event.payload, false);
    });
    const unlistenStopped = listen<unknown>(BROWSER_ENGINE_STOPPED_EVENT, (event) => {
      handleNativeEvent(event.payload, true);
    });
    return () => {
      cleanupTauriListener(unlistenReady);
      cleanupTauriListener(unlistenStopped);
    };
  }, [schedule]);

  useEffect(() => {
    const updateOcclusion = () => {
      const next = isBrowserNativeOccluded();
      if (occludedRef.current === next) return;
      occludedRef.current = next;
      if (next) hide();
      else schedule();
    };
    window.addEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, updateOcclusion);
    return () => window.removeEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, updateOcclusion);
  }, [hide, schedule]);

  useEffect(() => () => {
    if (frameRef.current !== null) window.cancelAnimationFrame(frameRef.current);
    hide();
  }, [hide]);

  return { hostRef, retrySurface };
}
