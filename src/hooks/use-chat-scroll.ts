import { useState, useCallback, useRef, useEffect, useLayoutEffect } from "react";

interface UseChatScrollReturn {
  containerRef: React.RefObject<HTMLDivElement | null>;
  isAtBottom: boolean;
  scrollToBottom: () => void;
}

export function useChatScroll(sessionId: string, isStreaming: boolean, deps: unknown[]): UseChatScrollReturn {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const autoScrollRef = useRef(true);

  useLayoutEffect(() => {
    let cancelled = false;
    autoScrollRef.current = true;
    queueMicrotask(() => {
      if (!cancelled) setIsAtBottom(true);
    });
    return () => { cancelled = true; };
  }, [sessionId]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        autoScrollRef.current = false;
        setIsAtBottom(false);
      } else if (e.deltaY > 0) {
        const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight <= 50;
        if (nearBottom) {
          autoScrollRef.current = true;
          setIsAtBottom(true);
        }
      }
    };
    el.addEventListener("wheel", onWheel, { passive: true });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  useLayoutEffect(() => {
    if (!autoScrollRef.current) return;
    const el = containerRef.current;
    if (el) el.scrollTop = el.scrollHeight;
    // eslint-disable-next-line react-hooks/exhaustive-deps -- deps is a passthrough for caller-controlled re-runs
  }, [sessionId, isStreaming, ...deps]);

  const scrollToBottom = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
    autoScrollRef.current = true;
    setIsAtBottom(true);
  }, []);

  return { containerRef, isAtBottom, scrollToBottom };
}
