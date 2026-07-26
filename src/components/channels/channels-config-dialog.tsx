import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import type { ChannelType } from "@/types/channels";
import { configureGatewayAccountTokens } from "./channels-config-api";

interface ChannelsConfigDialogProps {
  channelId: ChannelType;
  onClose: () => void;
  onSaved: (accountId: string) => void;
}

type TestState =
  | { kind: "idle" }
  | { kind: "testing" }
  | { kind: "ok" }
  | { kind: "error"; message: string };

export function ChannelsConfigDialog({ channelId, onClose, onSaved }: ChannelsConfigDialogProps) {
  const { t } = useTranslation();
  const [accountId, setAccountId] = useState("");
  const [token, setToken] = useState("");
  const [botToken, setBotToken] = useState("");
  const [appToken, setAppToken] = useState("");
  const [testState, setTestState] = useState<TestState>({ kind: "idle" });
  const [submitting, setSubmitting] = useState(false);
  const isSlack = channelId === "slack";

  const clearSecrets = useCallback(() => {
    setToken("");
    setBotToken("");
    setAppToken("");
  }, []);

  const handleClose = useCallback(() => {
    clearSecrets();
    onClose();
  }, [clearSecrets, onClose]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        handleClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [handleClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!accountId.trim() || !tokensReady(isSlack, token, botToken, appToken)) return;
    setSubmitting(true);
    setTestState({ kind: "testing" });
    try {
      const credentials = isSlack ? { botToken, appToken } : { token };
      await configureGatewayAccountTokens(channelId, accountId, credentials);
      setTestState({ kind: "ok" });
      setTimeout(() => onSaved(accountId.trim()), 500);
    } catch {
      setTestState({ kind: "error", message: t("channels.detail.testFailed") });
    } finally {
      clearSecrets();
      setSubmitting(false);
    }
  };

  const channelName = channelId.charAt(0).toUpperCase() + channelId.slice(1);
  const canSubmit = Boolean(accountId.trim() && tokensReady(isSlack, token, botToken, appToken));

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={handleClose} onKeyDown={() => {}}>
      <div className="wk-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <header className="wk-dialog-header">
          <span>{t("channels.config.addTitle", { name: channelName })}</span>
          <button type="button" className="wk-dialog-close" onClick={handleClose}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <form className="wk-form" onSubmit={(e) => void handleSubmit(e)}>
          <div className="wk-form-field">
            <label className="wk-form-label">{t("channels.detail.accountId")}</label>
            <input
              type="text"
              className="wk-input"
              value={accountId}
              onChange={(e) => setAccountId(e.target.value)}
              placeholder="my-bot"
              autoFocus
            />
          </div>

          {isSlack ? (
            <>
              <div className="wk-form-field">
                <label className="wk-form-label">{t("channels.detail.botToken")}</label>
                <input
                  type="password"
                  className="wk-input"
                  value={botToken}
                  onChange={(e) => setBotToken(e.target.value)}
                  placeholder={t("channels.detail.tokenPlaceholder")}
                />
              </div>
              <div className="wk-form-field">
                <label className="wk-form-label">{t("channels.detail.appToken")}</label>
                <input
                  type="password"
                  className="wk-input"
                  value={appToken}
                  onChange={(e) => setAppToken(e.target.value)}
                  placeholder={t("channels.detail.appTokenPlaceholder")}
                />
              </div>
            </>
          ) : (
            <div className="wk-form-field">
              <label className="wk-form-label">{t("channels.detail.token")}</label>
              <input
                type="password"
                className="wk-input"
                value={token}
                onChange={(e) => setToken(e.target.value)}
                placeholder={t("channels.detail.tokenPlaceholder")}
              />
            </div>
          )}

          <div className="wk-form-field">
            <label className="wk-form-label">{t("channels.config.description")}</label>
            <div className="wk-input ch-config-description">
              {t(`channels.browse.${channelId}Desc`)}
            </div>
          </div>

          {testState.kind === "testing" && (
            <div className="ak-test-result loading">{t("channels.config.testing")}</div>
          )}
          {testState.kind === "ok" && (
            <div className="ak-test-result success">{t("channels.detail.testSuccess")}</div>
          )}
          {testState.kind === "error" && (
            <div className="ak-test-result error">{testState.message}</div>
          )}

          <footer className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={handleClose} disabled={submitting}>
              {t("channels.detail.cancel")}
            </button>
            <button
              type="submit"
              className="wk-btn-primary"
              disabled={submitting || !canSubmit}
            >
              {t("channels.config.addAndTest")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}

function tokensReady(isSlack: boolean, token: string, botToken: string, appToken: string): boolean {
  return isSlack ? Boolean(botToken.trim() && appToken.trim()) : Boolean(token.trim());
}
