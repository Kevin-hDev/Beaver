import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ModelEditorShell } from "./model-editor-shell";
import "./ollama.css";

interface ModelfileEditorProps {
  modelName: string;
  initialContent: string;
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function ModelfileEditor({
  modelName, initialContent, onSave, onCancel,
}: ModelfileEditorProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_modelfile", { name: modelName, content });
      onSave(content);
    } catch {
      setError(t("errors.operationFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <ModelEditorShell
      title={`${modelName} — ${t("ollama.editing")}`}
      cancelLabel={t("ollama.cancel")}
      saveLabel={t("ollama.save")}
      saving={saving}
      error={error}
      fillAvailableSpace
      onCancel={onCancel}
      onSave={() => void handleSave()}
    >
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        aria-label={t("ollama.editing")}
        className="mes-textarea mfed-textarea"
      />
    </ModelEditorShell>
  );
}
