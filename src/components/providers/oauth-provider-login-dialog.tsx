import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/lib/provider-icons";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { OAuthLoginProgress, OAuthProviderStatus } from "@/types/oauth-provider";
import "@/components/connectors/mcp-oauth-dialog.css";

type LoginState = "loading" | "waiting" | "success" | "error";

interface OAuthProviderLoginDialogProps {
  provider: OAuthProviderStatus;
  onClose: () => void;
  onConnected: () => void;
}

export function OAuthProviderLoginDialog({ provider, onClose, onConnected }: OAuthProviderLoginDialogProps) {
  const { t } = useTranslation();
  const [state, setState] = useState<LoginState>("loading");
  const [userCode, setUserCode] = useState<string | null>(null);
  const [verificationUrl, setVerificationUrl] = useState<string | null>(null);
  const mountedRef = useRef(true);
  const onConnectedRef = useRef(onConnected);

  useEffect(() => {
    onConnectedRef.current = onConnected;
  }, [onConnected]);

  const startFlow = useCallback(async () => {
    setUserCode(null);
    setVerificationUrl(null);
    setState("loading");
    try {
      await invoke("cancel_oauth_provider_login", { providerId: provider.id });
      if (!mountedRef.current) return;
      await invoke("start_oauth_provider_login", {
        providerId: provider.id,
        diagnosticId: crypto.randomUUID(),
      });
    } catch {
      if (mountedRef.current) setState("error");
    }
  }, [provider.id]);

  useEffect(() => {
    mountedRef.current = true;
    let active = true;
    let successTimer: ReturnType<typeof setTimeout> | undefined;
    const unlisten = listen<OAuthLoginProgress>("oauth-login-progress", ({ payload }) => {
      if (!mountedRef.current || payload.provider_id !== provider.id) return;
      if (payload.stage === "success") {
        setState("success");
        successTimer = setTimeout(() => onConnectedRef.current(), 600);
      } else if (payload.stage === "cancelled" || payload.stage === "error") {
        setState("error");
      } else {
        setUserCode(payload.user_code ?? null);
        setVerificationUrl(payload.verification_url ?? null);
        setState("waiting");
      }
    });
    void unlisten.then(() => { if (active) void startFlow(); });
    return () => {
      active = false;
      mountedRef.current = false;
      if (successTimer) clearTimeout(successTimer);
      cleanupTauriListener(unlisten);
    };
  }, [provider.id, startFlow]);

  useEffect(() => {
    if (provider.connected) onConnectedRef.current();
  }, [provider.connected]);

  const handleClose = () => {
    void invoke("cancel_oauth_provider_login", { providerId: provider.id });
    onClose();
  };
  const message = userCode
    ? t("providers.oauth.deviceInstructions")
    : t(state === "waiting" ? "connectors.oauth.message" : state === "success" ? "providers.oauth.successMessage" : state === "error" ? "connectors.oauth.errorGeneric" : "connectors.oauth.discovering");

  const copyCode = async () => {
    if (!userCode) return;
    try {
      await navigator.clipboard.writeText(userCode);
    } catch {
      // Clipboard access can be denied by the operating system.
    }
  };

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={handleClose} onKeyDown={() => undefined}>
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="wk-dialog mco-dialog" role="dialog" onClick={(event) => event.stopPropagation()} onKeyDown={() => undefined}>
        <div className="mco-icons">
          <div className="mco-icon-box"><ProviderIcon providerId={provider.id} displayName={provider.display_name} size={40} /></div>
          <div className="mco-connector-line" />
          <div className="mco-icon-box"><span className="mco-app-icon">CL</span></div>
        </div>
        <h3 className="mco-title">{t(state === "success" ? "connectors.oauth.successTitle" : "connectors.oauth.title")}</h3>
        <p className={`mco-message ${state === "error" ? "mco-error" : ""}`}>{message}</p>
        {userCode && (
          <div className="mco-device-code">
            <span>{t("providers.oauth.codeLabel")}</span>
            <strong>{userCode}</strong>
            <button type="button" className="ollama-btn" onClick={() => void copyCode()}>{t("providers.oauth.copyCode")}</button>
          </div>
        )}
        {verificationUrl && state === "waiting" && (
          <button type="button" className="mco-retry-link" onClick={() => void open(verificationUrl)}>{t("providers.oauth.openVerification")}</button>
        )}
        {(state === "waiting" || state === "error") && (
          <button type="button" className="mco-retry-link" onClick={() => void startFlow()}>{t("connectors.oauth.retry")}</button>
        )}
        <div className="wk-dialog-footer mco-footer">
          <button type="button" className="wk-btn-secondary" onClick={handleClose}>{t("connectors.oauth.cancel")}</button>
        </div>
      </div>
    </div>
  );
}
