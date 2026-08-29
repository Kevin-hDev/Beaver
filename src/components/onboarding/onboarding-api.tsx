import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { ArrowSquareOut, CaretRight } from "@/components/ui/icons";
import { ApiKeySecretInput } from "@/components/api-keys/api-key-secret-input";
import { InlineToast } from "@/components/ui/toast";
import { showToast } from "@/lib/toast-emitter";
import type { ProviderSpec, QwenConnectionInput } from "@/types/api";
import {
  DEFAULT_QWEN_CONNECTION,
  isQwenConnectionValid,
  ProviderConnectionForm,
} from "@/components/api-keys/provider-connection-form";
import { OnboardingProviderGrid } from "./onboarding-provider-grid";

interface OnboardingApiProps {
  onComplete: () => void | Promise<void>;
  onBack: () => void;
}

type SaveState = "idle" | "saving" | "saved" | "error";

export function OnboardingApi({ onComplete, onBack }: OnboardingApiProps) {
  const { t } = useTranslation();
  const [providers, setProviders] = useState<ProviderSpec[]>([]);
  const [configuredIds, setConfiguredIds] = useState<string[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [connection, setConnection] = useState<QwenConnectionInput>(DEFAULT_QWEN_CONNECTION);

  useEffect(() => {
    Promise.all([
      invoke<ProviderSpec[]>("list_llm_providers_catalog"),
      invoke<string[]>("list_configured_providers"),
    ])
      .then(([items, configured]) => {
        const llmProviders = items.filter((item) => item.category === "llm").slice(0, 32);
        const displayedIds = new Set(llmProviders.map((provider) => provider.id));
        setProviders(llmProviders);
        setConfiguredIds(configured.filter((id) => displayedIds.has(id)));
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
    setConnection(DEFAULT_QWEN_CONNECTION);
  }, []);

  const handleSave = useCallback(async () => {
    const key = apiKey.trim();
    const requiresConnection = selected?.connection_kind === "qwen_model_studio";
    if (!selected || !key || (requiresConnection && !isQwenConnectionValid(connection))) return;
    setSaveState("saving");
    try {
      const providerConnection = requiresConnection ? connection : undefined;
      await invoke("test_api_key_with_value", {
        provider: selected.id, key, connection: providerConnection,
      });
      await invoke("set_api_key", { provider: selected.id, key, connection: providerConnection });
      setConfiguredIds((current) =>
        current.includes(selected.id) ? current : [...current, selected.id],
      );
      setApiKey("");
      setSaveState("saved");
      showToast(t("apiKeys.dialog.testOk"), "success");
    } catch {
      setSaveState("error");
    }
  }, [apiKey, connection, selected, t]);

  return (
    <div className="ob-page ob-page-api">
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.api.title")}</h1>
        <p className="ob-description">{t("onboarding.api.description")}</p>
      </div>

      <OnboardingProviderGrid
        providers={providers}
        configuredIds={configuredSet}
        selectedId={selectedId}
        onSelect={selectProvider}
      />

      <div className="ob-api-form">
        {selected?.connection_kind === "qwen_model_studio" && (
          <ProviderConnectionForm
            value={connection}
            onChange={setConnection}
            disabled={saveState === "saving"}
          />
        )}
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
          className="btn btn-sm btn-secondary"
          onClick={onBack}
          disabled={saveState === "saving"}
        >
          {t("onboarding.common.back")}
        </button>
        <button
          type="button"
          className="btn btn-sm btn-primary"
          onClick={() => void handleSave()}
          disabled={
            !selected
            || !apiKey.trim()
            || saveState === "saving"
            || (selected.connection_kind === "qwen_model_studio"
              && !isQwenConnectionValid(connection))
          }
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
          className="btn btn-sm btn-secondary"
          onClick={() => void finish()}
          disabled={saveState === "saving"}
          data-e2e="api-skip"
        >
          {t("onboarding.common.skip")}
        </button>
      </div>
    </div>
  );
}
