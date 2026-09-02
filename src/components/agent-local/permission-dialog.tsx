import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { PermissionRequest, PermissionDecision } from "@/hooks/use-permission-requests";
import "./permission-dialog.css";

interface Props {
  request: PermissionRequest;
  onDecide: (id: string, decision: PermissionDecision) => void;
}

function extractTarget(args: Record<string, unknown>): string {
  const keys = ["path", "command", "url"];
  for (const k of keys) {
    const v = args[k];
    if (typeof v === "string" && v.length > 0) return v;
  }
  return "";
}

export function PermissionDialog({ request, onDecide }: Props) {
  const { t } = useTranslation();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onDecide(request.id, "deny");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [request.id, onDecide]);

  const target = extractTarget(request.arguments);
  const externalRead = request.effectClass === "external-read";
  const allowSession = request.allowSession !== false;
  const action = externalRead
    ? t("permissionDialog.tools.web_fetch")
    : request.extensionName
      ? t("permissionDialog.extensionAction", { extension: request.extensionName })
      : t(`permissionDialog.tools.${request.toolName}`, { defaultValue: request.toolName });
  const title = t("permissionDialog.title", { action });

  return (
    <div className="perm-card" role="dialog" aria-modal="false">
      <div className="perm-card-title">{title}</div>
      {externalRead && request.extensionName && <div>{request.extensionName}</div>}
      {request.effectClass && (
        <div>{t(`permissionDialog.effects.${request.effectClass}`)}</div>
      )}
      {request.actionSummary && <pre className="perm-card-target">{request.actionSummary}</pre>}
      {target && <pre className="perm-card-target">{target}</pre>}
      <div className="perm-card-actions">
        <button
          type="button"
          className="btn btn-sm btn-ghost"
          onClick={() => onDecide(request.id, "deny")}
        >
          {t("permissionDialog.deny")}
          <span className="perm-card-kbd">esc</span>
        </button>
        <div className="perm-card-actions-spacer" />
        <button
          type="button"
          className="btn btn-sm btn-secondary"
          onClick={() => onDecide(request.id, "allow")}
          autoFocus={!allowSession}
        >
          {t("permissionDialog.allow")}
        </button>
        {allowSession && (
          <button
            type="button"
            className="btn btn-sm btn-primary"
            onClick={() => onDecide(request.id, "allow_session")}
            autoFocus
          >
            {t("permissionDialog.allowSession")}
          </button>
        )}
      </div>
    </div>
  );
}
