import { useState, type ReactNode } from "react";
import { CaretRight } from "@/components/ui/icons";
import { Collapsible } from "@/components/ui/collapsible";

interface CompressionSectionProps {
  title: string;
  note?: string;
  children: ReactNode;
  defaultOpen?: boolean;
}

export function CompressionSection({
  title,
  note,
  children,
  defaultOpen = true,
}: CompressionSectionProps) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section className="cse-section relief">
      <button
        type="button"
        className="cse-section-head"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <CaretRight className="cse-section-chevron" size="var(--icon-sm)" />
        <span>{title}</span>
        {note && <span className="cse-section-note">{note}</span>}
      </button>
      <Collapsible open={open} unmountWhenClosed>
        <div className="cse-section-body">{children}</div>
      </Collapsible>
    </section>
  );
}
