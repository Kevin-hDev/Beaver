import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import type { GithubBranchAuthState } from "@/hooks/use-github-branch-auth";

interface BranchGithubAuthDialogProps {
  state: GithubBranchAuthState;
  onCancel: () => void;
  onConnect: () => void;
}

export function BranchGithubAuthDialog({
  state,
  onCancel,
  onConnect,
}: BranchGithubAuthDialogProps) {
  const { t } = useTranslation();
  const busy = state === "connecting" || state === "testing";

  return (
    <div className="bcd-overlay" role="presentation" onClick={onCancel} onKeyDown={() => {}}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="bcd-dialog" role="dialog" onClick={(e) => e.stopPropagation()}>
        <button className="icon-btn bcd-close" onClick={onCancel} type="button">
          <X size="var(--icon-md)" />
        </button>
        <div className="bcd-title">{t("branches.githubAuthTitle")}</div>
        <div className="bcd-description">{t("branches.githubAuthDescription")}</div>
        {state === "connecting" && <div className="bcd-hint">{t("branches.githubAuthConnecting")}</div>}
        {state === "testing" && <div className="bcd-hint">{t("branches.githubAuthTesting")}</div>}
        {state === "error" && <div className="bcd-error">{t("branches.githubAuthError")}</div>}
        <div className="bcd-actions">
          <button className="bcd-btn" onClick={onCancel} type="button" disabled={busy}>
            {t("branches.githubAuthCancel")}
          </button>
          <button className="bcd-btn bcd-btn-primary" onClick={onConnect} type="button" disabled={busy}>
            {t("branches.githubAuthConnect")}
          </button>
        </div>
      </div>
    </div>
  );
}
