import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { CloseIcon, OpenExternalIcon } from "@/components/ui/panel-action-icons";
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
            <OpenExternalIcon />
          </button>
        </Tooltip>
        {hasAnalysis && (
          <Tooltip label={t("a11y.close")} align="right">
            <button className="icon-btn fp-icon-btn" onClick={onCloseAnalysis}>
              <CloseIcon />
            </button>
          </Tooltip>
        )}
      </div>
    </div>
  );
}
