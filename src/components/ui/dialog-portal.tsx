import type { ReactNode } from "react";
import { createPortal } from "react-dom";

interface DialogPortalProps {
  children: ReactNode;
}

export function DialogPortal({ children }: DialogPortalProps) {
  return createPortal(children, document.body);
}
