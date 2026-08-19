import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { Collapsible } from "@/components/ui/collapsible";
import "./messages.css";
import "./stream-active.css";

interface ThinkingSectionProps {
  content: string;
  durationMs?: number;
  isActive?: boolean;
}

export function ThinkingSection({ content, durationMs, isActive }: ThinkingSectionProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  if (!content) return null;

  const seconds = durationMs ? (durationMs / 1000).toFixed(1) : null;
  const label = seconds ? t("agentLocal.thinkingDuration", { seconds }) : t("agentLocal.thinking");
  const labelClass = `thinking-label${isActive ? " stream-active-label" : ""}`;

  return (
    <div>
      <button
        type="button"
        className="thinking-toggle"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        <span className={labelClass}>{label}</span>
        <span className="thinking-chevron" aria-hidden="true">
          {open ? <CaretUp size="var(--icon-sm)" weight="bold" /> : <CaretDown size="var(--icon-sm)" weight="bold" />}
        </span>
      </button>
      <Collapsible open={open}>
        <div className="thinking-content">{content}</div>
      </Collapsible>
    </div>
  );
}
