import { useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ChatInput } from "./chat-input";
import { WelcomeWordmark } from "./welcome-wordmark";
import { ProjectSelector } from "./project-selector";
import { FileDropZone } from "./file-drop-zone";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import type { Project } from "@/types/agent";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import { useDirectoryAccessGuard } from "@/hooks/use-directory-access-guard";
import { addProjectDirectory, selectProjectDirectory } from "@/hooks/project-directory-selection";
import { showToast } from "@/lib/toast-emitter";
import { noteComposerPosition, takeComposerPosition } from "@/lib/composer-handoff";
import { waitForTitleExit } from "./welcome-leave";
import { WELCOME_COMPOSER_DRAFT_KEY } from "@/hooks/use-composer-draft";
import "./welcome-view.css";

interface WelcomeViewProps {
  model: string;
  provider: string;
  projects: Project[];
  onAddProject: (path: string) => Promise<Project>;
  onSend: (text: string, files?: DroppedFile[], projectId?: string, skills?: { name: string; content: string }[]) => void | Promise<void>;
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
  const { prompt: directoryAccessPrompt, request: requestDirectoryAccess } = useDirectoryAccessGuard();
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [leaving, setLeaving] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  const handleAddProject = useCallback(
    () => addProjectDirectory(requestDirectoryAccess, onAddProject, (project) => setSelectedProjectId(project.id)),
    [onAddProject, requestDirectoryAccess],
  );

  const handleSelectProject = useCallback((id: string | null) => {
    selectProjectDirectory(id, projects, requestDirectoryAccess, setSelectedProjectId);
  }, [projects, requestDirectoryAccess]);

  const handleSend = useCallback((text: string, files?: DroppedFile[], skills?: { name: string; content: string }[]) => {
    const hasFiles = files && files.length > 0;
    if (!text.trim() && !hasFiles && (!skills || skills.length < 1)) return;
    const send = async () => {
      /* Le champ ne bouge plus d'ici : il note sa place et c'est celui de la
         conversation qui, en naissant, descendra depuis elle. Une distance
         parcourue à l'aveugle depuis cet écran ne pouvait être juste qu'à une
         seule taille de fenêtre. */
      const bubble = contentRef.current?.querySelector<HTMLElement>(".chat-input-bubble");
      if (bubble) noteComposerPosition(bubble.getBoundingClientRect().top);
      setLeaving(true);
      await waitForTitleExit(contentRef.current);
      try {
        await onSend(text, files, selectedProjectId ?? undefined, skills);
      } catch (error) {
        takeComposerPosition();
        setLeaving(false);
        throw error;
      }
    };
    const project = projects.find((candidate) => candidate.id === selectedProjectId);
    if (selectedProjectId && !project) {
      showToast(t("errors.operationFailed"), "error");
      return;
    }
    if (project) {
      void requestDirectoryAccess(project.path, send);
    } else {
      void send().catch(() => showToast(t("errors.operationFailed"), "error"));
    }
  }, [onSend, projects, requestDirectoryAccess, selectedProjectId, t]);

  return (
    <FileDropZone
      enabled
      dragging={fileDrop.dragging}
      onDragChange={fileDrop.setDragging}
      onDropPaths={(paths) => void fileDrop.addByPaths(paths)}
    >
      <div className="welcome-zone">
        <div className="welcome-content" ref={contentRef}>
          <WelcomeWordmark leaving={leaving} />
          <div className="welcome-input-wrap">
            <ChatInput
              draftKey={WELCOME_COMPOSER_DRAFT_KEY}
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
              onSelect={handleSelectProject}
              onAddProject={() => void handleAddProject()}
              directoryAccessPrompt={directoryAccessPrompt}
            />
          </div>
        </div>
      </div>
    </FileDropZone>
  );
}
