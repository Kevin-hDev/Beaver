import { useCallback, useEffect, useRef } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { SkillReference } from "@/types/agent-turn.generated";

interface ChatActionsOptions {
  readOnly: boolean;
  chat: {
    messages: { role: string; id: string }[];
    sendMessage: (text: string, files?: DroppedFile[], workingDir?: string, projectId?: string, skills?: SkillReference[]) => Promise<boolean>;
    reload: (id: string) => Promise<void>;
    isStreaming: boolean;
  };
  selectedProjectPath?: string;
  selectedProjectId?: string;
  onSessionsRefresh?: () => void;
  onAutoRename?: (id: string, name: string) => void;
  sessionId: string;
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkills?: SkillReference[];
  initialFiles?: DroppedFile[];
  onInitialMessageSent?: () => void;
  fileDrop: { addByPaths: (paths: string[]) => Promise<void> };
}

export function useChatActions({
  readOnly,
  chat, selectedProjectPath, selectedProjectId,
  onSessionsRefresh, onAutoRename, sessionId,
  initialMessage, initialWorkingDir, initialSkills, initialFiles,
  onInitialMessageSent, fileDrop,
}: ChatActionsOptions) {
  const initialSent = useRef(false);

  useEffect(() => {
    if (readOnly) return;
    const hasContent = initialMessage || (initialFiles && initialFiles.length > 0) || (initialSkills && initialSkills.length > 0);
    if (hasContent && !initialSent.current) {
      initialSent.current = true;
      const workingDir = initialWorkingDir ?? selectedProjectPath;
      const files = initialFiles?.map((file) => ({ ...file }));
      void chat.sendMessage(initialMessage || "", files, workingDir, selectedProjectId, initialSkills)
        .then((accepted) => { if (accepted) onInitialMessageSent?.(); });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- one-time send on mount
  }, [initialMessage, readOnly]);

  const handleSend = useCallback(async (
    text: string,
    sentFiles?: DroppedFile[],
    skills?: SkillReference[],
  ) => {
    if (readOnly) return false;
    const isFirst = chat.messages.length < 1;
    const accepted = await chat.sendMessage(
      text, sentFiles, selectedProjectPath, selectedProjectId, skills,
    );
    if (!accepted) return false;
    if (selectedProjectId) onSessionsRefresh?.();
    if (isFirst && text.trim()) {
      const autoName = text.slice(0, 40).trim();
      if (autoName) onAutoRename?.(sessionId, autoName);
    }
    return true;
  }, [chat, selectedProjectPath, selectedProjectId, onSessionsRefresh, onAutoRename, readOnly, sessionId]);

  const handleFileImport = useCallback(() => {
    if (readOnly) return;
    void (async () => {
      const result = await openFileDialog({ multiple: true });
      if (!result) return;
      const paths = (Array.isArray(result) ? result : [result]).map((p) => String(p));
      await fileDrop.addByPaths(paths);
    })();
  }, [fileDrop, readOnly]);

  return { handleSend, handleFileImport };
}
