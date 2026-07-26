import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { CustomSelect } from "@/components/ui/custom-select";
import type { CreateWakeupInput, ScheduledWakeup, WakeupSchedule } from "@/types/wakeup";
import { useAvailableModels, withoutInteractiveOnlyModels } from "@/hooks/use-available-models";
import { SchedulePicker } from "./schedule-picker";

interface NewWakeupDialogProps {
  initial: ScheduledWakeup | null;
  onClose: () => void;
  onCreate: (input: CreateWakeupInput) => Promise<void>;
  onUpdate: (wakeup: ScheduledWakeup) => Promise<void>;
}

function defaultSchedule(): WakeupSchedule {
  return { kind: "daily", time: "08:00" };
}

export function NewWakeupDialog({
  initial,
  onClose,
  onCreate,
  onUpdate,
}: NewWakeupDialogProps) {
  const { t } = useTranslation();
  const { groups } = useAvailableModels();
  const heartbeatGroups = useMemo(() => withoutInteractiveOnlyModels(groups), [groups]);
  const [name, setName] = useState(initial?.name ?? "");
  const [provider, setProvider] = useState(initial?.provider ?? "ollama");
  const [model, setModel] = useState(initial?.model ?? "");
  const [prompt, setPrompt] = useState(initial?.prompt ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [schedule, setSchedule] = useState<WakeupSchedule>(
    initial?.schedule ?? defaultSchedule(),
  );
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const availableProviders = useMemo(() => {
    return Array.from(heartbeatGroups.keys()).map((id) => ({
      id,
      display_name: heartbeatGroups.get(id)?.[0]?.provider_name ?? id,
    }));
  }, [heartbeatGroups]);

  const toolCapableModels = useMemo(() => {
    return (heartbeatGroups.get(provider) ?? []).filter((m) => m.supports_tools);
  }, [heartbeatGroups, provider]);

  useEffect(() => {
    if (!toolCapableModels.find((m) => m.id === model)) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- derived state reset when provider changes is intentional
      setModel(toolCapableModels[0]?.id ?? "");
    }
  }, [provider, toolCapableModels, model]);

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
    setSubmitting(true);
    setError(null);
    try {
      if (initial) {
        await onUpdate({ ...initial, name, provider, model, prompt, description, schedule });
      } else {
        await onCreate({ name, model, provider, prompt, description, schedule });
      }
      onClose();
    } catch (err) {
      console.warn("[wakeup create]", err);
      setError(t("errors.operationFailed"));
    } finally {
      setSubmitting(false);
    }
  };

  const disabled = submitting || toolCapableModels.length === 0;
  const title = initial ? t("heartbeat.form.editTitle") : t("heartbeat.form.createTitle");

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label="Close dialog" onClick={onClose} onKeyDown={(e) => { if (e.key === "Escape") onClose(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="wk-dialog" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="wk-dialog-header">
          <span>{title}</span>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <form className="wk-form" onSubmit={(e) => void handleSubmit(e)}>
          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.name")}</label>
            <input
              type="text"
              className="wk-input"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              autoFocus
            />
          </div>

          <div className="wk-form-row">
            <div className="wk-form-field">
              <label className="wk-form-label">{t("heartbeat.form.provider")}</label>
              <CustomSelect
                value={provider}
                onChange={setProvider}
                options={
                  availableProviders.length === 0
                    ? [{ value: "ollama", label: "Ollama" }]
                    : availableProviders.map((p) => ({ value: p.id, label: p.display_name }))
                }
              />
            </div>

            <div className="wk-form-field">
              <label className="wk-form-label">{t("heartbeat.form.model")}</label>
              <CustomSelect
                value={model}
                onChange={setModel}
                disabled={toolCapableModels.length === 0}
                placeholder={
                  toolCapableModels.length === 0
                    ? t("heartbeat.form.noToolCapable")
                    : t("heartbeat.form.pickModel")
                }
                options={toolCapableModels.map((m) => ({ value: m.id, label: m.id }))}
              />
            </div>
          </div>

          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.prompt")}</label>
            <textarea
              className="wk-input wk-textarea"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={4}
              required
            />
          </div>

          <SchedulePicker value={schedule} onChange={setSchedule} />

          <div className="wk-form-field">
            <label className="wk-form-label">{t("heartbeat.form.description")}</label>
            <input
              type="text"
              className="wk-input"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              maxLength={200}
            />
          </div>

          {error && <div className="wk-form-error">{error}</div>}

          <footer className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={onClose}>
              {t("heartbeat.form.cancel")}
            </button>
            <button type="submit" className="wk-btn-primary" disabled={disabled}>
              {initial ? t("heartbeat.form.save") : t("heartbeat.form.create")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
