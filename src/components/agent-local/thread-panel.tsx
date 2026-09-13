import type { ReactNode } from "react";
import { Collapsible } from "@/components/ui/collapsible";
import { ChevronDown } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import "./thread-panel.css";

export interface ThreadPanelProps {
  icon: ReactNode;
  title: ReactNode;
  headerExtra?: ReactNode;
  headerAction?: ReactNode;
  open: boolean;
  onToggle: () => void;
  className?: string;
  children: ReactNode;
}

/* Autorité visuelle commune des panneaux placés dans le fil. */
export function ThreadPanel({
  icon,
  title,
  headerExtra,
  headerAction,
  open,
  onToggle,
  className,
  children,
}: ThreadPanelProps) {
  return (
    <div className={cn("thp-root relief", className)}>
      <div className={cn("thp-header", headerAction && "thp-has-action")}>
        <button aria-expanded={open} className="thp-toggle" onClick={onToggle} type="button">
          <span className="thp-icon">{icon}</span>
          <span className="thp-title">{title}</span>
          <span className="thp-extra">{headerExtra}</span>
          <ChevronDown className={cn("thp-chevron", open && "thp-chevron-open")} aria-hidden="true" />
        </button>
        {headerAction && <span className="thp-action">{headerAction}</span>}
      </div>
      <Collapsible open={open}>
        <div className="thp-list">{children}</div>
      </Collapsible>
    </div>
  );
}
