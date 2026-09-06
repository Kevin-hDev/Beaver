import type { ReactNode } from "react";
import { AppSurfaceBoundary } from "@/components/layout/app-surface-boundary";

interface AgentLocalPanelSurfaceProps {
  children: ReactNode;
}

export function AgentLocalPanelSurface({ children }: AgentLocalPanelSurfaceProps) {
  return (
    <AppSurfaceBoundary className="al-panel-surface">
      {children}
    </AppSurfaceBoundary>
  );
}
