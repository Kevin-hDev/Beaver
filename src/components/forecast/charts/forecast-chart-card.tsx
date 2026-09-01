import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "@/components/ui/icons";
import "./forecast-chart-card.css";
import { Collapsible } from "@/components/ui/collapsible";

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
    // <Collapsible> anime `height`, et l'événement remonte depuis sa propre
    // région : la cible n'est donc pas ce conteneur, seule la propriété
    // permet de reconnaître la fin du dépliement.
    const handleTransitionEnd = (event: TransitionEvent) => {
      if (event.propertyName === "height") {
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
          className="icon-btn fcrd-toggle"
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
      {/* Replié, le corps reste dans la page pour pouvoir s'animer, mais il
          sort de l'ordre de tabulation et de l'arbre d'accessibilité. */}
      <div ref={bodyRef} className="fcrd-body" inert={!open} aria-hidden={!open}>
        <Collapsible open={open} innerClassName="fcrd-inner">
          {children}
        </Collapsible>
      </div>
    </section>
  );
}
