import { useState } from "react";
import { ChevronDown } from "@/components/ui/icons";
import { ForecastDocsMarkdown } from "./forecast-docs-markdown";
import type { ForecastDocSection } from "./forecast-docs-types";

interface ForecastDocsAccordionProps {
  section: ForecastDocSection;
  defaultOpen?: boolean;
}

export function ForecastDocsAccordion({
  section,
  defaultOpen = false,
}: ForecastDocsAccordionProps) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section className={`fd-accordion ${open ? "fd-accordion-open" : ""}`}>
      <button
        className="fd-accordion-trigger"
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-expanded={open}
        aria-controls={`fd-section-${section.id}`}
      >
        <span>{section.title}</span>
        <ChevronDown size="var(--icon-15)" className="fd-accordion-icon" />
      </button>
      <div id={`fd-section-${section.id}`} className="fd-accordion-panel">
        <div className="fd-accordion-content">
          <ForecastDocsMarkdown content={section.body} />
        </div>
      </div>
    </section>
  );
}
