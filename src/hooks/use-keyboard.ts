import { useEffect } from "react";

interface KeyboardHandlers {
  onEscape?: () => void;
  onEnter?: () => void;
}

const KEY_MAP: Record<string, keyof KeyboardHandlers> = {
  Escape: "onEscape",
  Enter: "onEnter",
};

const TEXTAREA_TAG = "TEXTAREA";

export function useKeyboard(handlers: KeyboardHandlers) {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const handlerName = KEY_MAP[e.key];
      if (!handlerName) return;

      const handler = handlers[handlerName];
      if (!handler) return;

      // Don't intercept Enter inside textareas
      if (handlerName.includes("Enter")) {
        const tag = (e.target as HTMLElement).tagName;
        if (tag.includes(TEXTAREA_TAG)) return;
      }

      e.preventDefault();
      handler();
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handlers]);
}
