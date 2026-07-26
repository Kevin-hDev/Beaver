import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { X, ArrowSquareOut } from "@/components/ui/icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";
import { ApiKeySecretInput } from "./api-key-secret-input";

interface ApiKeysConfigDialogProps {
  provider: ProviderSpec;
  /** true = provider déjà configuré (édition) ; false = ajout */
  alreadyConfigured: boolean;
  onClose: () => void;
  onSave: (key: string) => Promise<void>;
  onTest: (key: string) => Promise<void>;
  onClearKey?: () => Promise<void>;
}

type TestState =
  | { kind: "idle" }
  | { kind: "testing" }
  | { kind: "ok" }
  | { kind: "error"; message: string };

export function ApiKeysConfigDialog({
  provider,
  alreadyConfigured,
  onClose,
  onSave,
  onTest,
  onClearKey,
}: ApiKeysConfigDialogProps) {
  const { t, i18n } = useTranslation();
  const [apiKey, setApiKey] = useState("");
  const [testState, setTestState] = useState<TestState>({ kind: "idle" });
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!apiKey.trim()) return;
    setSubmitting(true);
    setTestState({ kind: "testing" });
    try {
      await onTest(apiKey.trim());
      setTestState({ kind: "ok" });
      await onSave(apiKey.trim());
      setTimeout(() => onClose(), 500);
    } catch {
      setTestState({ kind: "error", message: t("errors.operationFailed") });
    } finally {
      setSubmitting(false);
    }
  };

  const handleClearKey = async () => {
    if (!onClearKey) return;
    setSubmitting(true);
    try {
      await onClearKey();
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label="Close dialog" onClick={onClose} onKeyDown={(e) => { if (e.key === "Escape") onClose(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="wk-dialog" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="wk-dialog-header">
          <span>
            {alreadyConfigured
              ? t("apiKeys.dialog.editTitle", { name: provider.display_name })
              : t("apiKeys.dialog.addTitle", { name: provider.display_name })}
          </span>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <form className="wk-form" onSubmit={(e) => void handleSubmit(e)}>
          <div className="wk-form-field">
            <label className="wk-form-label">{t("apiKeys.dialog.apiKey")}</label>
            <ApiKeySecretInput
              inputClassName="wk-input"
              value={apiKey}
              onChange={setApiKey}
              placeholder={
                alreadyConfigured
                  ? t("apiKeys.dialog.keyPlaceholderEdit")
                  : t("apiKeys.dialog.keyPlaceholder")
              }
              autoFocus
              required={!alreadyConfigured}
            />
            <button
              type="button"
              className="ak-signup-inline"
              onClick={() => void open(provider.signup_url)}
            >
              {t("apiKeys.dialog.getKeyAt", { name: provider.display_name })}
              <ArrowSquareOut size="var(--icon-xs)" />
            </button>
          </div>

          <div className="wk-form-field">
            <label className="wk-form-label">{t("apiKeys.dialog.description")}</label>
            <div className="wk-input" style={{ cursor: "default", opacity: 0.7 }}>
              {getProviderDescription(provider, i18n.language)}
            </div>
          </div>

          {testState.kind === "testing" && (
            <div className="ak-test-result loading">{t("apiKeys.dialog.testing")}</div>
          )}
          {testState.kind === "ok" && (
            <div className="ak-test-result success">{t("apiKeys.dialog.testOk")}</div>
          )}
          {testState.kind === "error" && (
            <div className="ak-test-result error">{testState.message}</div>
          )}

          <footer className="wk-dialog-footer">
            {alreadyConfigured && onClearKey && (
              <button
                type="button"
                className="ak-btn-danger-outline"
                onClick={() => void handleClearKey()}
                disabled={submitting}
              >
                {t("apiKeys.dialog.clearKey")}
              </button>
            )}
            <button
              type="button"
              className="wk-btn-secondary"
              onClick={onClose}
              disabled={submitting}
            >
              {t("apiKeys.dialog.cancel")}
            </button>
            <button
              type="submit"
              className="wk-btn-primary"
              disabled={submitting || !apiKey.trim()}
            >
              {alreadyConfigured
                ? t("apiKeys.dialog.save")
                : t("apiKeys.dialog.addAndTest")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
