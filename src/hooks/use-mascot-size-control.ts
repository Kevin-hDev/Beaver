import { useCallback, useEffect, useRef, useState } from "react";
import {
  MASCOT_SIZE_SAVE_DELAY_MS,
  normalizeMascotSize,
} from "@/services/mascot";

type PersistMascotSize = (sizePercent: number) => Promise<void>;

export function useMascotSizeControl(
  persistedSize: number,
  persistSize: PersistMascotSize,
) {
  const [sizePercent, setSizePercent] = useState(persistedSize);
  const persistedRef = useRef(persistedSize);
  const latestRef = useRef(persistedSize);
  const persistRef = useRef(persistSize);
  const timerRef = useRef<number | null>(null);
  const generationRef = useRef(0);
  const pendingRef = useRef(false);
  const mountedRef = useRef(true);

  useEffect(() => {
    persistRef.current = persistSize;
  }, [persistSize]);

  useEffect(() => {
    persistedRef.current = persistedSize;
    if (!pendingRef.current) {
      latestRef.current = persistedSize;
      setSizePercent(persistedSize);
    }
  }, [persistedSize]);

  const commit = useCallback((value: number, generation: number) => {
    timerRef.current = null;
    void persistRef.current(value).then(() => {
      if (!mountedRef.current || generation !== generationRef.current) return;
      pendingRef.current = false;
      setSizePercent(value);
    }).catch(() => {
      if (!mountedRef.current || generation !== generationRef.current) return;
      pendingRef.current = false;
      latestRef.current = persistedRef.current;
      setSizePercent(persistedRef.current);
    });
  }, []);

  const changeSize = useCallback((value: number) => {
    const next = normalizeMascotSize(value);
    const generation = ++generationRef.current;
    latestRef.current = next;
    pendingRef.current = true;
    setSizePercent(next);
    if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(
      () => commit(next, generation),
      MASCOT_SIZE_SAVE_DELAY_MS,
    );
  }, [commit]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (timerRef.current === null) return;
      window.clearTimeout(timerRef.current);
      void persistRef.current(latestRef.current).catch(() => {});
    };
  }, []);

  return { sizePercent, changeSize };
}
