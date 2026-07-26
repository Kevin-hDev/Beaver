import { useCallback, useEffect, useRef } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface ChatActionsOptions {
  chat: {
    messages: { role: string; id: string }[];
    sendMessage: (text: string, files?: DroppedFile[], workingDir?: string, projectId?: string, skills?: { name: string; content: string }[]) => Promise<void>;
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
  initialSkills?: { name: string; content: string }[];
  initialFiles?: DroppedFile[];
  onInitialMessageSent?: () => void;
  fileDrop: { addByPaths: (paths: string[]) => Promise<void> };
}

export function useChatActions({
  chat, selectedProjectPath, selectedProjectId,
  onSessionsRefresh, onAutoRename, sessionId,
  initialMessage, initialWorkingDir, initialSkills, initialFiles,
  onInitialMessageSent, fileDrop,
}: ChatActionsOptions) {
  const initialSent = useRef(false);

  useEffect(() => {
    const hasContent = initialMessage || (initialFiles && initialFiles.length > 0) || (initialSkills && initialSkills.length > 0);
    if (hasContent && !initialSent.current) {
      initialSent.current = true;
      const workingDir = initialWorkingDir ?? selectedProjectPath;
      const files = initialFiles?.map((file) => ({ ...file }));
      void chat.sendMessage(initialMessage || "", files, workingDir, selectedProjectId, initialSkills)
        .then(() => onInitialMessageSent?.());
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- one-time send on mount
  }, [initialMessage]);

  const handleSend = useCallback((
    text: string,
    sentFiles?: DroppedFile[],
    skills?: { name: string; content: string }[],
  ) => {
    const isFirst = chat.messages.length < 1;
    void chat.sendMessage(text, sentFiles, selectedProjectPath, selectedProjectId, skills)
      .then(() => {
        if (selectedProjectId) onSessionsRefresh?.();
        if (isFirst && text.trim()) {
          const autoName = text.slice(0, 40).trim();
          if (autoName) onAutoRename?.(sessionId, autoName);
        }
      });
  }, [chat, selectedProjectPath, selectedProjectId, onSessionsRefresh, onAutoRename, sessionId]);

  const handleFileImport = useCallback(() => {
    void (async () => {
      const result = await openFileDialog({ multiple: true });
      if (!result) return;
      const paths = (Array.isArray(result) ? result : [result]).map((p) => String(p));
      await fileDrop.addByPaths(paths);
    })();
  }, [fileDrop]);

  return { handleSend, handleFileImport };
}
