import { useState } from "react";
import type { TransitionEvent } from "react";

export function useCollapsiblePresence(initialOpen = false) {
  const [open, setOpen] = useState(initialOpen);
  const [mounted, setMounted] = useState(initialOpen);

  const toggle = () => {
    if (!open) {
      setMounted(true);
      setOpen(true);
      return;
    }
    setOpen(false);
  };

  const onTransitionEnd = (event: TransitionEvent<HTMLElement>) => {
    if (event.currentTarget !== event.target) return;
    if (!open) setMounted(false);
  };

  return { mounted, onTransitionEnd, open, toggle };
}
