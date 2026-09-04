import { useState, useCallback, useEffect, useRef } from "react";
import { migrateAppNav, type AppNavPatch, type AppNavState } from "@/types/navigation";
import {
  CORE_NAVIGATION_AVAILABILITY,
  type NavigationAvailability,
} from "@/features/extension-ui/slot-navigation";

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

export function useTabHistory(
  initial: AppNavState,
  availability: NavigationAvailability = CORE_NAVIGATION_AVAILABILITY,
) {
  const [current, setCurrent] = useState(() => migrateAppNav(initial, availability));
  const [navIndex, setNavIndex] = useState(0);
  const history = useRef<AppNavState[]>([current]);
  const live = useRef(current);

  useEffect(() => {
    setNavIndex((index) => {
      history.current = history.current.map((state) => migrateAppNav(state, availability));
      const next = history.current[index];
      live.current = next;
      setCurrent(next);
      return index;
    });
  }, [availability]);

  const pushNav = useCallback((partial: AppNavPatch) => {
    const next = migrateAppNav(mergePatch(live.current, partial), availability);
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
  }, [availability]);

  const replaceNav = useCallback((partial: AppNavPatch) => {
    const next = migrateAppNav(mergePatch(live.current, partial), availability);
    if (sameNav(live.current, next)) return;
    live.current = next;
    setNavIndex((i) => {
      history.current[i] = next;
      return i;
    });
    setCurrent(next);
  }, [availability]);

  const goBack = useCallback(() => {
    setNavIndex((i) => {
      if (i <= 0) return i;
      const newIdx = i - 1;
      const state = migrateAppNav(history.current[newIdx], availability);
      history.current[newIdx] = state;
      live.current = state;
      setCurrent(state);
      return newIdx;
    });
  }, [availability]);

  const goForward = useCallback(() => {
    setNavIndex((i) => {
      if (i >= history.current.length - 1) return i;
      const newIdx = i + 1;
      const state = migrateAppNav(history.current[newIdx], availability);
      history.current[newIdx] = state;
      live.current = state;
      setCurrent(state);
      return newIdx;
    });
  }, [availability]);

  const canGoBack = navIndex > 0;
  // eslint-disable-next-line react-hooks/refs -- derived from navIndex state change
  const canGoForward = navIndex < history.current.length - 1;

  return { current, pushNav, replaceNav, goBack, goForward, canGoBack, canGoForward };
}
