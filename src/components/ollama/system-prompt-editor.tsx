import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ModelEditorShell } from "./model-editor-shell";
import "./ollama.css";

interface SystemPromptEditorProps {
  modelName: string;
  initialSystem: string;
  onSave: (system: string) => void;
  onCancel: () => void;
}

export function SystemPromptEditor({
  modelName, initialSystem, onSave, onCancel,
}: SystemPromptEditorProps) {
  const { t } = useTranslation();
  const [system, setSystem] = useState(initialSystem);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_system_prompt", { name: modelName, system });
      onSave(system);
    } catch (e: unknown) {
      console.warn("[system-prompt save]", e);
      setError(t("errors.operationFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <ModelEditorShell
      title={`${modelName} — ${t("ollama.systemPrompt")}`}
      cancelLabel={t("ollama.cancel")}
      saveLabel={t("ollama.save")}
      saving={saving}
      fillAvailableSpace
      error={error}
      onCancel={onCancel}
      onSave={() => void handleSave()}
    >
      <textarea
        value={system}
        onChange={(e) => setSystem(e.target.value)}
        placeholder={t("ollama.systemPromptPlaceholder")}
        aria-label={t("ollama.systemPrompt")}
        className="mes-textarea sped-textarea"
      />
    </ModelEditorShell>
  );
}
