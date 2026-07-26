import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

interface BrowserReplaceDialogProps {
  candidateTitle: string;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function BrowserReplaceDialog(props: BrowserReplaceDialogProps) {
  const { t } = useTranslation();
  const title = props.candidateTitle || t("browser.newTab");
  return createPortal(
    <div className="ib-dialog-overlay" role="presentation">
      <section className="ib-dialog" role="dialog" aria-modal="true" aria-labelledby="ib-dialog-title">
        <h2 id="ib-dialog-title">{t("browser.tabLimitTitle")}</h2>
        <p>{t("browser.tabLimitDescription", { title })}</p>
        <div className="ib-dialog-actions">
          <button type="button" className="ib-dialog-button" disabled={props.busy} onClick={props.onCancel}>
            {t("common.cancel")}
          </button>
          <button type="button" className="ib-dialog-button ib-dialog-confirm" disabled={props.busy} onClick={props.onConfirm}>
            {t("browser.replaceTab")}
          </button>
        </div>
      </section>
    </div>,
    document.body,
  );
}
