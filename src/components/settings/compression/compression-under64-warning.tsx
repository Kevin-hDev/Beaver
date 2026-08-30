import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "@/components/ui/toggle-switch";

interface CompressionUnder64WarningProps {
  toggleVisible: boolean;
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}

export function CompressionUnder64Warning({
  toggleVisible,
  enabled,
  onChange,
}: CompressionUnder64WarningProps) {
  const { t } = useTranslation();
  if (!toggleVisible && !enabled) return null;

  return (
    <>
      {toggleVisible && <div className="cse-under64 relief">
        <span className="cse-under64-copy">
          <span className="cse-row-title">
            {t("settings.advanced.compressionUnder64Title")}
          </span>
          <span className="cse-row-desc">
            {t("settings.advanced.compressionUnder64Desc")}
          </span>
          <span className="cse-under64-state">
            {t(enabled
              ? "settings.advanced.compressionStateEnabled"
              : "settings.advanced.compressionStateDisabled")}
          </span>
        </span>
        <ToggleSwitch
          checked={enabled}
          onCheckedChange={onChange}
          ariaLabel={t("settings.advanced.compressionUnder64Title")}
        />
      </div>}
      {enabled && (
        <div className="cse-warning" role="status">
          {t("settings.advanced.compressionUnder64Warning")}
        </div>
      )}
    </>
  );
}
