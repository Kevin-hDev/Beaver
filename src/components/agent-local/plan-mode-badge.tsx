import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import "./plan-mode-badge.css";

interface PlanModeBadgeProps {
  onDisable: () => void;
}

export function PlanModeBadge({ onDisable }: PlanModeBadgeProps) {
  const { t } = useTranslation();
  return (
    <Tooltip label={t("chatMenu.disablePlanMode")}>
      <button
        type="button"
        className="pmb-root"
        onClick={onDisable}
        aria-label={t("chatMenu.disablePlanMode")}
      >
        <span className="pmb-text">{t("chatMenu.planMode")}</span>
        <X size="var(--icon-2xs)" className="pmb-x" />
      </button>
    </Tooltip>
  );
}
