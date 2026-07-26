import type { ReactNode } from "react";
import "./inline-activity-indicator.css";

interface InlineActivityIndicatorProps {
  children: ReactNode;
  className?: string;
}

export function InlineActivityIndicator({ children, className = "" }: InlineActivityIndicatorProps) {
  return (
    <span className={`iai-indicator ${className}`}>
      <span className="iai-bars" aria-hidden="true">
        <span />
        <span />
        <span />
        <span />
      </span>
      <span className="iai-label">{children}</span>
    </span>
  );
}
