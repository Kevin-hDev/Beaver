import { useState, useEffect, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ModelfileEditor } from "./modelfile-editor";
import { ParametersEditor } from "./parameters-editor";
import { ModelfileView } from "./modelfile-view";
import { extractParameters } from "./modelfile-utils";
import { cleanupTauriListener } from "@/lib/tauri-listen";

type Mode = "view" | "edit-parameters" | "edit-modelfile";

interface ModelfileViewerProps {
  modelName: string;
  onDeleted?: () => void;
}

export function ModelfileViewer({ modelName, onDeleted }: ModelfileViewerProps) {
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

  if (loading) {
    return (
      <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
        {t("history.loading")}
      </div>
    );
  }

  if (mode === "edit-parameters") {
    return (
      <ParametersEditor
        modelName={modelName}
        initialParameters={parameters}
        onSave={() => { setMode("view"); void loadModelfile(); }}
        onCancel={() => setMode("view")}
      />
    );
  }

  if (mode === "edit-modelfile") {
    return (
      <ModelfileEditor
        modelName={modelName}
        initialContent={modelfile}
        onSave={(c) => { setModelfile(c); setMode("view"); }}
        onCancel={() => setMode("view")}
      />
    );
  }

  return (
    <ModelfileView
      modelName={modelName}
      parameters={parameters}
      modelfile={modelfile}
      deleting={deleting}
      onDelete={() => void handleDelete()}
      onEditParameters={() => setMode("edit-parameters")}
      onEditModelfile={() => setMode("edit-modelfile")}
    />
  );
}
