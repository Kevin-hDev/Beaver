import { useState, useCallback, useRef } from "react";
import { migrateAppNav, type AppNavPatch, type AppNavState } from "@/types/navigation";

const MAX_HISTORY = 50;

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function mergePatch<T>(base: T, patch: AppNavPatch): T {
  const result: Record<string, unknown> = { ...(base as Record<string, unknown>) };
  for (const [key, value] of Object.entries(patch)) {
    const current = result[key];
    result[key] = isPlainObject(current) && isPlainObject(value) && !("kind" in value)
      ? mergePatch(current, value as AppNavPatch)
      : value;
  }
  return result as T;
}

function sameNav(a: AppNavState, b: AppNavState): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function useTabHistory(initial: AppNavState) {
  const [current, setCurrent] = useState(() => migrateAppNav(initial));
  const [navIndex, setNavIndex] = useState(0);
  const history = useRef<AppNavState[]>([current]);
  const live = useRef(current);

  const pushNav = useCallback((partial: AppNavPatch) => {
    const next = mergePatch(live.current, partial);
    if (sameNav(live.current, next)) return;
    live.current = next;
    setNavIndex((i) => {
      history.current = history.current.slice(0, i + 1);
      history.current.push(next);
      if (history.current.length > MAX_HISTORY) {
        history.current.shift();
        return i;
      }
      return i + 1;
    });
    setCurrent(next);
  }, []);

  const replaceNav = useCallback((partial: AppNavPatch) => {
    const next = mergePatch(live.current, partial);
    if (sameNav(live.current, next)) return;
    live.current = next;
    setNavIndex((i) => {
      history.current[i] = next;
      return i;
    });
    setCurrent(next);
  }, []);

  const goBack = useCallback(() => {
    setNavIndex((i) => {
      if (i <= 0) return i;
      const newIdx = i - 1;
      const state = history.current[newIdx];
      live.current = state;
      setCurrent(state);
      return newIdx;
    });
  }, []);

  const goForward = useCallback(() => {
    setNavIndex((i) => {
      if (i >= history.current.length - 1) return i;
      const newIdx = i + 1;
      const state = history.current[newIdx];
      live.current = state;
      setCurrent(state);
      return newIdx;
    });
  }, []);

  const canGoBack = navIndex > 0;
  // eslint-disable-next-line react-hooks/refs -- derived from navIndex state change
  const canGoForward = navIndex < history.current.length - 1;

  return { current, pushNav, replaceNav, goBack, goForward, canGoBack, canGoForward };
}
