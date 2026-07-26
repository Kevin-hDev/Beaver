import { useCallback, useRef, type RefCallback } from "react";

export function useHoverClass(className = "msg-hovered"): RefCallback<HTMLDivElement> {
  const cleanupRef = useRef<(() => void) | null>(null);

  return useCallback((element) => {
    cleanupRef.current?.();
    cleanupRef.current = null;
    if (!element) return;

    const onEnter = () => element.classList.add(className);
    const onLeave = () => element.classList.remove(className);

    element.addEventListener("mouseenter", onEnter);
    element.addEventListener("mouseleave", onLeave);
    cleanupRef.current = () => {
      element.removeEventListener("mouseenter", onEnter);
      element.removeEventListener("mouseleave", onLeave);
      element.classList.remove(className);
    };
  }, [className]);
}
