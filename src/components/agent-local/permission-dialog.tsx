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
  const action = t(`permissionDialog.tools.${request.toolName}`, { defaultValue: request.toolName });
  const title = t("permissionDialog.title", { action });

  return (
    <div className="perm-card" role="dialog" aria-modal="false">
      <div className="perm-card-title">{title}</div>
      {target && <pre className="perm-card-target">{target}</pre>}
      <div className="perm-card-actions">
        <button
          type="button"
          className="perm-card-btn perm-card-btn-deny"
          onClick={() => onDecide(request.id, "deny")}
        >
          {t("permissionDialog.deny")}
          <span className="perm-card-kbd">esc</span>
        </button>
        <div className="perm-card-actions-spacer" />
        <button
          type="button"
          className="perm-card-btn"
          onClick={() => onDecide(request.id, "allow")}
        >
          {t("permissionDialog.allow")}
        </button>
        <button
          type="button"
          className="perm-card-btn perm-card-btn-primary"
          onClick={() => onDecide(request.id, "allow_session")}
          autoFocus
        >
          {t("permissionDialog.allowSession")}
        </button>
      </div>
    </div>
  );
}
