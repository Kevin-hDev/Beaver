import type { ReactNode } from "react";
import { CaretDown } from "@/components/ui/icons";

interface SessionSummarySectionProps {
  icon: ReactNode;
  title: string;
  count: number;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function SessionSummarySection({
  icon,
  title,
  count,
  open,
  onToggle,
  children,
}: SessionSummarySectionProps) {
  return (
    <section className="ssb-section">
      <button className="ssb-section-toggle" type="button" aria-expanded={open} onClick={onToggle}>
        <span className="ssb-section-label">
          <span className="ssb-section-icon">{icon}</span>
          <span>{title} ({count})</span>
        </span>
        <CaretDown
          className={`ssb-section-caret ${open ? "ssb-section-caret-open" : ""}`}
          size="var(--icon-sm)"
        />
      </button>
      <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
        <div className="ssb-accordion-inner">{children}</div>
      </div>
    </section>
  );
}
