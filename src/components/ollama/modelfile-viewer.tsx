import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { SettingsDetailHeader } from "@/components/settings/shell/settings-detail-header";
import { ModelfileEditor } from "./modelfile-editor";
import { ParametersEditor } from "./parameters-editor";
import { ModelfileView } from "./modelfile-view";
import { extractParameters } from "./modelfile-utils";
import { cleanupTauriListener } from "@/lib/tauri-listen";

type Mode = "view" | "edit-parameters" | "edit-modelfile";

interface ModelfileViewerProps {
  modelName: string;
  onBack: () => void;
  onDeleted?: () => void;
}

export function ModelfileViewer({ modelName, onBack, onDeleted }: ModelfileViewerProps) {
  const { t } = useTranslation();
  const [modelfile, setModelfile] = useState("");
  const [mode, setMode] = useState<Mode>("view");
  const [loading, setLoading] = useState(true);
  const [deleting, setDeleting] = useState(false);

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await invoke("delete_ollama_model", { name: modelName });
      onDeleted?.();
    } catch (e: unknown) {
      console.warn("[ollama] delete model:", e);
    } finally {
      setDeleting(false);
    }
  };

  const parameters = useMemo(() => extractParameters(modelfile), [modelfile]);

  const loadModelfile = useCallback(() => {
    return invoke<string>("get_modelfile", { name: modelName })
      .then(setModelfile)
      .catch(() => {});
  }, [modelName]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- model switch resets local editor state before async reload
    setLoading(true);
    setMode("view");
    void loadModelfile().finally(() => setLoading(false));
  }, [loadModelfile]);

  useEffect(() => {
    const unlisten = listen("modelfile-updated", () => { void loadModelfile(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [loadModelfile]);

  /* Les actions portent sur le modèle nommé juste à côté : les poser ailleurs
     que dans son en-tête laisse « Supprimer » sans sujet visible. */
  const headerActions = mode === "view" && !loading ? (
    <>
      <ConfirmButton
        className="btn btn-sm btn-destructive"
        label={t("ollama.remove")}
        confirmLabel={t("settings.confirm.deleteModel")}
        onConfirm={() => void handleDelete()}
        disabled={deleting}
      />
      <button className="btn btn-sm btn-secondary" onClick={() => setMode("edit-modelfile")}>
        {t("ollama.editModelfile")}
      </button>
    </>
  ) : undefined;

  const body = loading ? (
    <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
      {t("history.loading")}
    </div>
  ) : mode === "edit-parameters" ? (
    <ParametersEditor
      modelName={modelName}
      initialParameters={parameters}
      onSave={() => { setMode("view"); void loadModelfile(); }}
      onCancel={() => setMode("view")}
    />
  ) : mode === "edit-modelfile" ? (
    <ModelfileEditor
      modelName={modelName}
      initialContent={modelfile}
      onSave={(c) => { setModelfile(c); setMode("view"); }}
      onCancel={() => setMode("view")}
    />
  ) : (
    <ModelfileView
      modelName={modelName}
      parameters={parameters}
      modelfile={modelfile}
      onEditParameters={() => setMode("edit-parameters")}
    />
  );

  return (
    <>
      <SettingsDetailHeader title={modelName} onBack={onBack} actions={headerActions} />
      {body}
    </>
  );
}
