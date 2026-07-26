import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { ArrowSquareOut, CaretRight, Check } from "@/components/ui/icons";
import { ApiKeySecretInput } from "@/components/api-keys/api-key-secret-input";
import { InlineToast } from "@/components/ui/toast";
import { showToast } from "@/lib/toast-emitter";
import { ProviderIcon } from "@/lib/provider-icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";

interface OnboardingApiProps {
  onComplete: () => void | Promise<void>;
  onBack: () => void;
}

type SaveState = "idle" | "saving" | "saved" | "error";

export function OnboardingApi({ onComplete, onBack }: OnboardingApiProps) {
  const { t, i18n } = useTranslation();
  const [providers, setProviders] = useState<ProviderSpec[]>([]);
  const [configuredIds, setConfiguredIds] = useState<string[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [saveState, setSaveState] = useState<SaveState>("idle");

  useEffect(() => {
    Promise.all([
      invoke<ProviderSpec[]>("list_llm_providers_catalog"),
      invoke<string[]>("list_configured_providers"),
    ])
      .then(([items, configured]) => {
        const llmProviders = items.filter((item) => item.category === "llm").slice(0, 32);
        setProviders(llmProviders);
        setConfiguredIds(configured);
        setSelectedId((current) => current || llmProviders[0]?.id || "");
      })
      .catch(() => {
        setProviders([]);
        setConfiguredIds([]);
      });
  }, []);

  const selected = useMemo(
    () => providers.find((provider) => provider.id === selectedId) ?? null,
    [providers, selectedId],
  );
  const configuredSet = useMemo(() => new Set(configuredIds), [configuredIds]);
  const selectedConfigured = selected ? configuredSet.has(selected.id) : false;

  const finish = useCallback(async () => {
    setApiKey("");
    await onComplete();
  }, [onComplete]);

  const selectProvider = useCallback((providerId: string) => {
    setSelectedId(providerId);
    setApiKey("");
    setSaveState("idle");
  }, []);

  const handleSave = useCallback(async () => {
    const key = apiKey.trim();
    if (!selected || !key) return;
    setSaveState("saving");
    try {
      await invoke("test_api_key_with_value", { provider: selected.id, key });
      await invoke("set_api_key", { provider: selected.id, key });
      setConfiguredIds((current) =>
        current.includes(selected.id) ? current : [...current, selected.id],
      );
      setApiKey("");
      setSaveState("saved");
      showToast(t("apiKeys.dialog.testOk"), "success");
    } catch {
      setSaveState("error");
    }
  }, [apiKey, selected, t]);

  return (
    <div className="ob-page ob-page-api">
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.api.title")}</h1>
        <p className="ob-description">{t("onboarding.api.description")}</p>
      </div>

      <div className="ob-provider-grid">
        {providers.length === 0 ? (
          <div className="ob-provider-empty">{t("onboarding.api.loading")}</div>
        ) : providers.map((provider) => {
          const isConfigured = configuredSet.has(provider.id);
          return (
          <button
            key={provider.id}
            type="button"
            className={[
              "ob-provider-card",
              provider.id === selectedId ? "is-active" : "",
              isConfigured ? "is-configured" : "",
            ].filter(Boolean).join(" ")}
            onClick={() => selectProvider(provider.id)}
          >
            <ProviderIcon
              providerId={provider.id}
              displayName={provider.display_name}
              size={28}
            />
            <span className="ob-provider-name">{provider.display_name}</span>
            <span className="ob-provider-desc">
              {getProviderDescription(provider, i18n.language)}
            </span>
            {isConfigured && (
              <span className="ob-provider-status">
                <Check size="var(--icon-xs)" weight="bold" />
                {t("apiKeys.details.connected")}
              </span>
            )}
          </button>
          );
        })}
      </div>

      <div className="ob-api-form">
        <div className="ob-api-heading">
          <label className="ob-field-label" htmlFor="ob-api-key">
            {selected
              ? t("onboarding.api.keyLabel", { name: selected.display_name })
              : t("onboarding.api.keyLabelFallback")}
          </label>
          {saveState === "error" && (
            <InlineToast type="error" compact className="ob-api-error">
              {t("errors.operationFailed")}
            </InlineToast>
          )}
        </div>
        <ApiKeySecretInput
          key={selected?.id ?? "empty"}
          id="ob-api-key"
          inputClassName="ob-api-input"
          value={apiKey}
          onChange={(value) => {
            setApiKey(value);
            setSaveState("idle");
          }}
          placeholder={
            selectedConfigured
              ? t("apiKeys.dialog.keyPlaceholderEdit")
              : t("onboarding.api.keyPlaceholder")
          }
          disabled={!selected || saveState === "saving"}
        />
        {selected && (
          <button
            type="button"
            className="ob-link-btn"
            onClick={() => void open(selected.signup_url)}
          >
            {t("onboarding.api.getKey", { name: selected.display_name })}
            <ArrowSquareOut size="var(--icon-13)" />
          </button>
        )}
        {saveState === "saved" && (
          <div className="ob-test-result success">{t("apiKeys.dialog.testOk")}</div>
        )}
      </div>

      <div className="ob-actions">
        <button
          type="button"
          className="ob-secondary-btn"
          onClick={onBack}
          disabled={saveState === "saving"}
        >
          {t("onboarding.common.back")}
        </button>
        <button
          type="button"
          className="ob-primary-btn"
          onClick={() => void handleSave()}
          disabled={!selected || !apiKey.trim() || saveState === "saving"}
        >
          {saveState === "saving"
            ? t("onboarding.api.saving")
            : selectedConfigured
              ? t("apiKeys.dialog.save")
              : t("apiKeys.dialog.addAndTest")}
          <CaretRight size="var(--icon-sm)" weight="bold" />
        </button>
        <button
          type="button"
          className="ob-secondary-btn"
          onClick={() => void finish()}
          disabled={saveState === "saving"}
        >
          {t("onboarding.common.skip")}
        </button>
      </div>
    </div>
  );
}
