import { Archive } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";

export function ContextCompressionMarker() {
  const { t } = useTranslation();
  return (
    <div className="msg-context-compressed" aria-label={t("agentLocal.contextCompressed")}>
      <Archive size="var(--icon-sm)" aria-hidden="true" />
      <span>{t("agentLocal.contextCompressed")}</span>
    </div>
  );
}
