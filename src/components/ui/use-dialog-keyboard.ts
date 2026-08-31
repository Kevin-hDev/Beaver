import { useEffect, useRef } from "react";

const FOCUSABLE = [
  "button:not([disabled]):not([tabindex='-1'])",
  "input:not([disabled]):not([tabindex='-1'])",
  "select:not([disabled]):not([tabindex='-1'])",
  "textarea:not([disabled]):not([tabindex='-1'])",
  "[href]:not([tabindex='-1'])",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

interface DialogKeyboardOptions {
  rootRef: React.RefObject<HTMLElement | null>;
  initialFocusRef: React.RefObject<HTMLElement | null>;
  onEscape: () => void;
  enabled?: boolean;
}

export function useDialogKeyboard({
  rootRef,
  initialFocusRef,
  onEscape,
  enabled = true,
}: DialogKeyboardOptions) {
  const initialFocusApplied = useRef(false);
  useEffect(() => {
    if (!enabled) return;
    if (!initialFocusApplied.current) {
      initialFocusApplied.current = true;
      initialFocusRef.current?.focus();
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onEscape();
        return;
      }
      if (event.key !== "Tab" || !rootRef.current) return;
      const items = [...rootRef.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (event.shiftKey && (active === first || !rootRef.current.contains(active))) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [enabled, initialFocusRef, onEscape, rootRef]);
}
