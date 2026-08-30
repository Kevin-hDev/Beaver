import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Info, X } from "@/components/ui/icons";
import { CustomSelect } from "@/components/ui/custom-select";
import type { CreateWakeupInput, ScheduledWakeup, WakeupSchedule } from "@/types/wakeup";
import { useAvailableModels, withoutInteractiveOnlyModels } from "@/hooks/use-available-models";
import { useProjects } from "@/hooks/use-projects";
import { SchedulePicker } from "./schedule-picker";
import { WakeupField, WakeupModelFields } from "./wakeup-form-fields";
import "./new-wakeup-dialog.css";

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
  const { projects } = useProjects();
  const heartbeatGroups = useMemo(() => withoutInteractiveOnlyModels(groups), [groups]);
  const [name, setName] = useState(initial?.name ?? "");
  const [provider, setProvider] = useState(initial?.provider ?? "ollama");
  const [model, setModel] = useState(initial?.model ?? "");
  const [prompt, setPrompt] = useState(initial?.prompt ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [projectId, setProjectId] = useState(initial?.project_id ?? "");
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
        await onUpdate({
          ...initial,
          name,
          provider,
          model,
          prompt,
          description,
          schedule,
          project_id: projectId || undefined,
        });
      } else {
        await onCreate({
          name,
          model,
          provider,
          prompt,
          description,
          schedule,
          project_id: projectId || undefined,
        });
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
      <div className="wk-dialog wk-dialog-wide" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="wk-dialog-header">
          <span>{title}</span>
          <button
            type="button"
            className="icon-btn icon-btn-secondary"
            onClick={onClose}
            aria-label={t("a11y.close")}
          >
            <X size="var(--icon-md)" />
          </button>
        </header>

        <form className="wk-form nwd-form" onSubmit={(e) => void handleSubmit(e)}>
          {/* Un réveil ne se déclenche pas machine éteinte ni application fermée :
              dit ici, où il se crée, plutôt que découvert le lendemain matin. */}
          <p className="nwd-notice">
            <Info size="var(--icon-sm)" weight="regular" />
            {t("heartbeat.form.notice")}
          </p>

          <WakeupField label={t("heartbeat.form.name")} required>
            <input
              type="text"
              className="field field-wide"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("heartbeat.form.namePlaceholder")}
              required
              autoFocus
            />
          </WakeupField>

          <WakeupField label={t("heartbeat.form.description")}>
            <input
              type="text"
              className="field field-wide"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t("heartbeat.form.descriptionPlaceholder")}
              maxLength={200}
            />
          </WakeupField>

          <WakeupModelFields
            provider={provider}
            model={model}
            providers={availableProviders}
            models={toolCapableModels}
            onProviderChange={setProvider}
            onModelChange={setModel}
          />

          <WakeupField label={t("heartbeat.form.project")}>
            <CustomSelect
              value={projectId}
              onChange={setProjectId}
              ariaLabel={t("heartbeat.form.project")}
              options={[
                { value: "", label: t("heartbeat.form.beaverWorkspace") },
                ...projects.map((project) => ({ value: project.id, label: project.name })),
              ]}
            />
          </WakeupField>

          <WakeupField label={t("heartbeat.form.prompt")} required>
            <textarea
              className="field field-wide field-multiline nwd-textarea"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder={t("heartbeat.form.promptPlaceholder")}
              rows={5}
              required
            />
          </WakeupField>

          <SchedulePicker value={schedule} onChange={setSchedule} />

          {error && <div className="wk-form-error">{error}</div>}

          <footer className="wk-dialog-footer">
            <button type="button" className="btn btn-sm btn-secondary" onClick={onClose}>
              {t("heartbeat.form.cancel")}
            </button>
            <button type="submit" className="btn btn-sm btn-primary" disabled={disabled}>
              {initial ? t("heartbeat.form.save") : t("heartbeat.form.create")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}
