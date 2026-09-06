import type { ReactNode } from "react";
import { createPortal } from "react-dom";
import { AppSurfaceBoundary } from "@/components/layout/app-surface-boundary";
import "./app-surface-portal.css";

interface AppSurfacePortalProps {
  children: ReactNode;
  target?: Element | DocumentFragment;
}

export function AppSurfacePortal({ children, target }: AppSurfacePortalProps) {
  return createPortal(
    <AppSurfaceBoundary className="app-surface-portal-boundary">
      {children}
    </AppSurfaceBoundary>,
    target ?? document.body,
  );
}
