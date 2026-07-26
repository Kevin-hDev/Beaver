import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AgentSessionMeta, Project } from "@/types/agent";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface ProjectsHookRef {
  projects: Project[];
}

interface CreateFn {
  (name: string, model: string, provider?: string, projectId?: string, reasoningMode?: string | null, supportsThinking?: boolean): Promise<AgentSessionMeta>;
}

interface RenameFn {
  (id: string, name: string): Promise<void>;
}

export interface SessionActionsDeps {
  create: CreateFn;
  rename: RenameFn;
  defaultModel: string;
  defaultProvider: string;
  welcomeModel: { model: string; provider: string } | null;
  setWelcomeModel: (v: { model: string; provider: string } | null) => void;
  welcomeReasoningMode?: string | null;
  welcomeSupportsThinking?: boolean;
  projectsHook: ProjectsHookRef;
  onSessionChange?: (id: string | null) => void;
}

export function useSessionActions(deps: SessionActionsDeps) {
  const { t } = useTranslation();
  const {
    create,
    rename,
    defaultModel,
    defaultProvider,
    welcomeModel,
    setWelcomeModel,
    welcomeReasoningMode,
    welcomeSupportsThinking,
    projectsHook,
    onSessionChange,
  } = deps;

  const [pendingMessage, setPendingMessage] = useState<string | null>(null);
  const [pendingWorkingDir, setPendingWorkingDir] = useState<string | undefined>(undefined);
  const [pendingSkills, setPendingSkills] = useState<{ name: string; content: string }[] | undefined>(undefined);
  const [pendingFiles, setPendingFiles] = useState<DroppedFile[] | undefined>(undefined);

  const handleCreate = useCallback(() => {
    onSessionChange?.(null);
  }, [onSessionChange]);

  const handleCreateWithModel = useCallback(
    async (newModel: string, newProvider: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, newModel, newProvider);
      onSessionChange?.(session.id);
    },
    [create, t, onSessionChange],
  );

  const handleWelcomeSend = useCallback(
    async (text: string, files?: DroppedFile[], projectId?: string, skills?: { name: string; content: string }[]) => {
      const name = text.slice(0, 40).trim() || t("agentLocal.newSession");
      const m = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
      const project = projectId ? projectsHook.projects.find((p) => p.id === projectId) : undefined;
      const session = await create(
        name,
        m.model,
        m.provider,
        projectId,
        welcomeReasoningMode,
        welcomeSupportsThinking,
      );
      setPendingMessage(text);
      setPendingWorkingDir(project?.path);
      setPendingSkills(skills);
      setPendingFiles(files && files.length > 0 ? files : undefined);
      setWelcomeModel(null);
      onSessionChange?.(session.id);
    },
    [create, defaultModel, defaultProvider, welcomeModel, setWelcomeModel, welcomeReasoningMode, welcomeSupportsThinking, t, projectsHook.projects, onSessionChange],
  );

  const handleAutoRename = useCallback(
    async (id: string, name: string) => {
      await rename(id, name);
    },
    [rename],
  );

  const handleCreateInProject = useCallback(
    async (projectId: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, defaultModel, defaultProvider, projectId);
      onSessionChange?.(session.id);
    },
    [create, defaultModel, defaultProvider, t, onSessionChange],
  );

  const handleCreateInProjectWithModel = useCallback(
    async (newModel: string, newProvider: string, projectId: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, newModel, newProvider, projectId);
      onSessionChange?.(session.id);
    },
    [create, t, onSessionChange],
  );

  return {
    pendingMessage,
    setPendingMessage,
    pendingWorkingDir,
    setPendingWorkingDir,
    pendingSkills,
    setPendingSkills,
    pendingFiles,
    setPendingFiles,
    handleCreate,
    handleCreateWithModel,
    handleWelcomeSend,
    handleAutoRename,
    handleCreateInProject,
    handleCreateInProjectWithModel,
  };
}
