import type React from "react";
import { CaretDown } from "@/components/ui/icons";
import { Collapsible } from "@/components/ui/collapsible";

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
      <Collapsible open={open} innerClassName="fca-accordion-content">
        {children}
      </Collapsible>
    </section>
  );
}
