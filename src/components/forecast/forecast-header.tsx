import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ArrowSquareOut } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastNav } from "./forecast-nav";

interface ForecastHeaderProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  hasAnalysis: boolean;
  contextLabel?: string | null;
  filterSlot?: ReactNode;
  rightSlot?: ReactNode;
  onToggleNav: () => void;
  onSectionChange: (section: ForecastSection) => void;
  onCloseAnalysis: () => void;
  onOpenWorkbench: () => void;
}

export function ForecastHeader({
  activeSection,
  navOpen,
  hasAnalysis,
  contextLabel,
  filterSlot,
  rightSlot,
  onToggleNav,
  onSectionChange,
  onCloseAnalysis,
  onOpenWorkbench,
}: ForecastHeaderProps) {
  const { t } = useTranslation();
  return (
    <div className="fc-head">
      <div className="fc-head-left">
        {activeSection === "view" && filterSlot}
        {hasAnalysis && (
          <ForecastNav
            open={navOpen}
            activeSection={activeSection}
            onToggle={onToggleNav}
            onSelect={onSectionChange}
          />
        )}
        {contextLabel && <span className="fc-context-label">{contextLabel}</span>}
      </div>
      <div className="fc-head-actions">
        {rightSlot}
        <Tooltip label={t("forecast.workbench.open")} align="right">
          <button
            className="icon-btn fp-icon-btn"
            type="button"
            aria-label={t("forecast.workbench.open")}
            onClick={onOpenWorkbench}
          >
            <ArrowSquareOut size="var(--icon-md)" />
          </button>
        </Tooltip>
        {hasAnalysis && (
          <Tooltip label={t("a11y.close")} align="right">
            <button className="icon-btn fp-icon-btn" onClick={onCloseAnalysis}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none"
                stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <path d="M3 3l8 8M11 3l-8 8" />
              </svg>
            </button>
          </Tooltip>
        )}
      </div>
    </div>
  );
}
