import type { ReactNode } from "react";
import { AppSurfacePortal } from "./app-surface-portal";

interface DialogPortalProps {
  children: ReactNode;
}

export function DialogPortal({ children }: DialogPortalProps) {
  return <AppSurfacePortal>{children}</AppSurfacePortal>;
}
