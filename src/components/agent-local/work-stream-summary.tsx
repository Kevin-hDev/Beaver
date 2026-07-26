import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretRight } from "@/components/ui/icons";
import { useCollapsiblePresence } from "./use-collapsible-presence";
import "./tool-bubble.css";
import "./work-stream-summary.css";

function formatWorkDuration(ms?: number): string | null {
  if (!ms || ms <= 0) return null;
  const secs = Math.max(0, Math.floor(ms / 1000));
  if (secs < 60) return `${secs} s`;
  const mins = Math.floor(secs / 60);
  const rest = secs % 60;
  if (mins < 60) return rest > 0 ? `${mins} min ${rest} s` : `${mins} min`;
  const hours = Math.floor(mins / 60);
  const remMins = mins % 60;
  return remMins > 0 ? `${hours} h ${remMins} min` : `${hours} h`;
}

export function WorkStreamSummary({
  children,
  defaultOpen = false,
  durationMs,
}: {
  children: ReactNode;
  defaultOpen?: boolean;
  durationMs?: number;
}) {
  const { t } = useTranslation();
  const { open, mounted, toggle, onTransitionEnd } = useCollapsiblePresence(defaultOpen);
  const duration = formatWorkDuration(durationMs);
  const label = duration
    ? t("agentLocal.workSummary", { duration })
    : t("agentLocal.workSummaryNoDuration");

  return (
    <div className="wss-root">
      <div className="wss-header">
        <button type="button" className="wss-toggle" aria-expanded={open} onClick={toggle}>
          <span>{label}</span>
          <span className="wss-chevron" aria-hidden="true">
            {open ? <CaretDown size="var(--icon-sm)" weight="bold" /> : <CaretRight size="var(--icon-sm)" weight="bold" />}
          </span>
        </button>
      </div>
      <div className={`tb-accordion${open ? " tb-open" : ""}`} onTransitionEnd={onTransitionEnd}>
        {mounted && <div className="tb-accordion-inner wss-body">{children}</div>}
      </div>
    </div>
  );
}
