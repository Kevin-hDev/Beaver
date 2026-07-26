import type React from "react";
import { CaretDown } from "@/components/ui/icons";

interface ForecastAnalysisAccordionProps {
  title: string;
  subtitle?: string;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

export function ForecastAnalysisAccordion({
  title,
  subtitle,
  open,
  onToggle,
  children,
}: ForecastAnalysisAccordionProps) {
  return (
    <section className={`fca-accordion ${open ? "is-open" : ""}`}>
      <button className="fca-accordion-head" type="button" onClick={onToggle}>
        <span>
          <span className="fca-accordion-title">{title}</span>
          {subtitle && <span className="fca-accordion-subtitle">{subtitle}</span>}
        </span>
        <CaretDown size="var(--icon-sm)" className="fca-accordion-caret" />
      </button>
      <div className="fca-accordion-panel">
        <div className="fca-accordion-content">{children}</div>
      </div>
    </section>
  );
}
