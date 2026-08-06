import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SettingsCard } from "@/components/settings/settings-card";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type {
  SystemPromptMode,
  SystemPromptTarget,
  SystemPromptTier,
  SystemPromptView,
} from "@/types/system-prompts";
import { SystemPromptEditorCard } from "./system-prompt-editor-card";
import { SystemPromptActions } from "./system-prompt-actions";
import { SystemPromptSelectors } from "./system-prompt-selectors";
import {
  shouldShowSystemPromptWarning,
  SystemPromptWarningDialog,
  type SystemPromptWarningKind,
} from "./system-prompt-warning-dialog";
import "./system-prompt-settings.css";

interface SystemPromptSettingsPanelProps {
  target: SystemPromptTarget;
  warningKind: SystemPromptWarningKind;
  initialMode: SystemPromptMode;
  initialTier: SystemPromptTier;
  selectorHeader?: ReactNode;
  selectorActions?: ReactNode;
}

export function SystemPromptSettingsPanel({
  target,
  warningKind,
  initialMode,
  initialTier,
  selectorHeader,
  selectorActions,
}: SystemPromptSettingsPanelProps) {
  const { t } = useTranslation();
  const [mode, setMode] = useState(initialMode);
  const [tier, setTier] = useState(initialTier);
  const [view, setView] = useState<SystemPromptView | null>(null);
  const [editing, setEditing] = useState(false);
  const [warning, setWarning] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(false);
  const loadSequence = useRef(0);
  const targetModel = target.scope === "ollama" ? target.model : null;
  const commandTarget = useMemo<SystemPromptTarget>(
    () => targetModel ? { scope: "ollama", model: targetModel } : { scope: "global" },
    [targetModel],
  );
  const targetKey = targetModel ? `ollama:${targetModel}` : "global";

  const load = useCallback(async () => {
    const sequence = ++loadSequence.current;
    try {
      const result = await invoke<SystemPromptView>("get_system_prompt_setting", {
        target: commandTarget,
        mode,
        tier,
      });
      if (sequence !== loadSequence.current) return;
      setView(result);
      setError(false);
    } catch {
      if (sequence !== loadSequence.current) return;
      setError(true);
    }
  }, [commandTarget, mode, tier]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- chargement IPC lié à la sélection active
    void load();
  }, [load]);
  useEffect(() => {
    const listeners = [listen("system-prompts-changed", () => { void load(); })];
    if (targetModel) {
      listeners.push(listen("modelfile-updated", () => { void load(); }));
    }
    return () => {
      for (const unlisten of listeners) cleanupTauriListener(unlisten);
    };
  }, [load, targetModel]);

  const startEditing = () => {
    if (shouldShowSystemPromptWarning(warningKind)) {
      setWarning(true);
    } else {
      setEditing(true);
    }
  };

  const selectMode = (nextMode: SystemPromptMode) => {
    if (saving || nextMode === mode) return;
    loadSequence.current += 1;
    setEditing(false);
    setView(null);
    setError(false);
    setMode(nextMode);
  };

  const selectTier = (nextTier: SystemPromptTier) => {
    if (saving || nextTier === tier) return;
    loadSequence.current += 1;
    setEditing(false);
    setView(null);
    setError(false);
    setTier(nextTier);
  };

  const save = async (content: string) => {
    setSaving(true);
    setError(false);
    try {
      const saved = await invoke<SystemPromptView>("save_system_prompt_setting", {
        target: commandTarget,
        mode,
        tier,
        content,
      });
      setView(saved);
      setEditing(false);
    } catch {
      setError(true);
    } finally {
      setSaving(false);
    }
  };

  const restore = async () => {
    setSaving(true);
    setError(false);
    try {
      const restored = await invoke<SystemPromptView>("restore_system_prompt_setting", {
        target: commandTarget,
        mode,
        tier,
      });
      setView(restored);
    } catch {
      setError(true);
    } finally {
      setSaving(false);
    }
  };

  const selectOllama = async () => {
    setSaving(true);
    setError(false);
    try {
      const restored = await invoke<SystemPromptView>("restore_default_system_prompt_setting", {
        target: commandTarget,
        mode,
        tier,
      });
      setView(restored);
    } catch {
      setError(true);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="spp-root">
      <SystemPromptSelectors
        mode={mode}
        tier={tier}
        onModeChange={selectMode}
        onTierChange={selectTier}
        header={selectorHeader}
        actions={selectorActions}
      />
      {editing && view ? (
        <SystemPromptEditorCard
          key={`${targetKey}:${mode}:${tier}`}
          initialContent={view.content}
          saving={saving}
          error={error}
          onCancel={() => { setEditing(false); setError(false); }}
          onSave={(content) => { void save(content); }}
        />
      ) : (
        <SettingsCard className="spp-card">
          <div className="spp-card-header">
            <div className="spp-title-group">
              <span className="spp-card-title">{t("settings.systemPrompt.instructions")}</span>
              {view && (
                <span className="spp-source">
                  {t(`settings.systemPrompt.sources.${view.source}`)}
                </span>
              )}
            </div>
            <SystemPromptActions
              view={view}
              isOllama={targetModel !== null}
              saving={saving}
              onUseBeaver={() => { void restore(); }}
              onUseOllama={() => { void selectOllama(); }}
              onEdit={startEditing}
            />
          </div>
          {error && <div className="spp-error" role="alert">{t("errors.operationFailed")}</div>}
          <PromptPreview view={view} emptyLabel={t("settings.systemPrompt.empty")} />
        </SettingsCard>
      )}
      {warning && (
        <SystemPromptWarningDialog
          kind={warningKind}
          onCancel={() => setWarning(false)}
          onContinue={() => { setWarning(false); setEditing(true); }}
        />
      )}
    </div>
  );
}

function PromptPreview({ view, emptyLabel }: { view: SystemPromptView | null; emptyLabel: string }) {
  if (!view) return <div className="spp-preview spp-preview-empty">…</div>;
  if (!view.content) return <div className="spp-preview spp-preview-empty">{emptyLabel}</div>;
  const lines = view.content.split("\n");
  const preview = lines.slice(0, 22).join("\n");
  return <div className="spp-preview">{preview}{lines.length > 22 ? "\n…" : ""}</div>;
}
