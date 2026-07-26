import type { ReactNode } from "react";

interface CollapsePanelProps {
  open: boolean;
  children: ReactNode;
}

export function CollapsePanel({ open, children }: CollapsePanelProps) {
  return (
    <div
      className={`conv-collapse-panel${open ? " conv-collapse-panel-open" : ""}`}
      aria-hidden={!open}
      inert={!open}
    >
      <div className="conv-collapse-content">{children}</div>
    </div>
  );
}
