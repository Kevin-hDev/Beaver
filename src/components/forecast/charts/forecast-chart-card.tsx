import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "@/components/ui/icons";
import "./forecast-chart-card.css";

const EXPAND_TRANSITION_MS = 380;

interface ForecastChartCardProps {
  title: string;
  defaultOpen?: boolean;
  /** Fired once the expand transition has finished (charts resize then). */
  onExpanded?: () => void;
  headerCenter?: ReactNode;
  children: ReactNode;
}

export function ForecastChartCard({
  title,
  defaultOpen = true,
  onExpanded,
  headerCenter,
  children,
}: ForecastChartCardProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(defaultOpen);
  const bodyRef = useRef<HTMLDivElement | null>(null);
  const wasOpenRef = useRef(defaultOpen);
  const onExpandedRef = useRef(onExpanded);

  useEffect(() => {
    onExpandedRef.current = onExpanded;
  }, [onExpanded]);

  useEffect(() => {
    if (!open) {
      wasOpenRef.current = false;
      return undefined;
    }
    // Only a real collapsed -> expanded transition notifies, not the mount.
    if (wasOpenRef.current) return undefined;
    wasOpenRef.current = true;
    const body = bodyRef.current;
    let notified = false;
    const notify = () => {
      if (notified) return;
      notified = true;
      onExpandedRef.current?.();
    };
    const timer = window.setTimeout(notify, EXPAND_TRANSITION_MS);
    const handleTransitionEnd = (event: TransitionEvent) => {
      if (event.target === body && event.propertyName === "grid-template-rows") {
        window.clearTimeout(timer);
        notify();
      }
    };
    body?.addEventListener("transitionend", handleTransitionEnd);
    return () => {
      window.clearTimeout(timer);
      body?.removeEventListener("transitionend", handleTransitionEnd);
    };
  }, [open]);

  return (
    <section className={`fcrd-card ${open ? "is-open" : ""}`}>
      <div className="fcrd-bar">
        <span className="fcrd-title">{title}</span>
        {headerCenter ? <div className="fcrd-center">{headerCenter}</div> : null}
        <button
          type="button"
          className="fcrd-toggle"
          aria-label={t(open ? "forecast.chartCard.collapse" : "forecast.chartCard.expand")}
          aria-expanded={open}
          onClick={() => setOpen((current) => !current)}
        >
          <ChevronDown
            size="var(--icon-sm)"
            className={`fcrd-chevron ${open ? "is-open" : ""}`}
          />
        </button>
      </div>
      <div
        ref={bodyRef}
        className={`fcrd-body ${open ? "is-open" : ""}`}
        // Collapsed bodies stay in the DOM for the grid-rows animation but
        // must leave the tab order and the accessibility tree.
        inert={!open}
        aria-hidden={!open}
      >
        <div className="fcrd-inner">{children}</div>
      </div>
    </section>
  );
}
