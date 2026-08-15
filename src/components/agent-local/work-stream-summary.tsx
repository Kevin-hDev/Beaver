import { useState } from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretRight } from "@/components/ui/icons";
import { Collapsible } from "@/components/ui/collapsible";
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
  const [open, setOpen] = useState(defaultOpen);
  const duration = formatWorkDuration(durationMs);
  const label = duration
    ? t("agentLocal.workSummary", { duration })
    : t("agentLocal.workSummaryNoDuration");

  return (
    <div className="wss-root">
      <div className="wss-header">
        <button
          type="button"
          className="wss-toggle"
          aria-expanded={open}
          onClick={() => setOpen((value) => !value)}
        >
          <span>{label}</span>
          <span className="wss-chevron" aria-hidden="true">
            {open ? <CaretDown size="var(--icon-xs)" weight="bold" /> : <CaretRight size="var(--icon-xs)" weight="bold" />}
          </span>
        </button>
      </div>
      <Collapsible open={open} unmountWhenClosed innerClassName="wss-body">
        {children}
      </Collapsible>
    </div>
  );
}
