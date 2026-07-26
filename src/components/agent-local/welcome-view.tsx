import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ChatInput } from "./chat-input";
import { ProjectSelector } from "./project-selector";
import { FileDropZone } from "./file-drop-zone";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import type { Project } from "@/types/agent";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import "./welcome-view.css";

interface WelcomeViewProps {
  model: string;
  provider: string;
  projects: Project[];
  onAddProject: (path: string) => Promise<Project>;
  onSend: (text: string, files?: DroppedFile[], projectId?: string, skills?: { name: string; content: string }[]) => void;
  onModelChange: (model: string, provider: string) => void;
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
}

export function WelcomeView({
  model, provider, projects, onAddProject, onSend, onModelChange, reasoningMode, onReasoningModeChange,
}: WelcomeViewProps) {
  const { t } = useTranslation();
  const permMode = usePermissionMode();
  const fileDrop = useFileDrop();
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [leaving, setLeaving] = useState(false);

  const handleAddProject = useCallback(async () => {
    const result = await openFileDialog({ directory: true });
    if (!result) return;
    const path = typeof result === "string" ? result : String(result);
    const project = await onAddProject(path);
    setSelectedProjectId(project.id);
  }, [onAddProject]);

  const handleSend = useCallback((text: string, files?: DroppedFile[], skills?: { name: string; content: string }[]) => {
    const hasFiles = files && files.length > 0;
    if (!text.trim() && !hasFiles && (!skills || skills.length < 1)) return;
    setLeaving(true);
    setTimeout(() => {
      onSend(text, files, selectedProjectId ?? undefined, skills);
    }, 350);
  }, [onSend, selectedProjectId]);

  return (
    <FileDropZone
      dragging={fileDrop.dragging}
      onDragChange={fileDrop.setDragging}
      onDropPaths={(paths) => void fileDrop.addByPaths(paths)}
    >
      <div className={`welcome-zone ${leaving ? "welcome-leaving" : ""}`}>
        <div className="welcome-content">
          <h1 className={`welcome-title ${leaving ? "welcome-title-leave" : ""}`}>
            {t("welcome.title")}
          </h1>
          <div className={`welcome-input-wrap ${leaving ? "welcome-input-leave" : ""}`}>
            <ChatInput
              modelName={model}
              providerName={provider}
              isStreaming={false}
              reasoningMode={reasoningMode}
              files={fileDrop.files}
              contextUsed={0}
              contextMax={0}
              permissionMode={permMode.mode}
              onPermissionModeChange={(m) => void permMode.change(m)}
              onSend={handleSend}
              onStop={() => {}}
              onRemoveFile={fileDrop.removeFile}
              onClearFiles={fileDrop.clearFiles}
              onFileImport={() => void (async () => {
                const result = await openFileDialog({ multiple: true });
                if (!result) return;
                const raw = Array.isArray(result) ? result : [result];
                await fileDrop.addByPaths(raw.map((p) => String(p)));
              })()}
              onModelChange={onModelChange}
              onReasoningModeChange={onReasoningModeChange}
            />
            <ProjectSelector
              projects={projects}
              selectedProjectId={selectedProjectId}
              locked={false}
              hidden={false}
              onSelect={setSelectedProjectId}
              onAddProject={() => void handleAddProject()}
            />
          </div>
        </div>
      </div>
    </FileDropZone>
  );
}
