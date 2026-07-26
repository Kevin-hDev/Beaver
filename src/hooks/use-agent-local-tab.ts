import { useState, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";
import { useDefaultModel } from "@/hooks/use-default-model";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useSessionActions } from "@/hooks/use-session-actions";
import { useFilePreview } from "@/hooks/use-file-preview";
import { useAgentLocalShortcuts } from "@/hooks/use-agent-local-shortcuts";
import { useAgentLocalTabPanelSync } from "@/hooks/use-agent-local-tab-panel-sync";
import { useAgentLocalControlledPreview } from "@/hooks/use-agent-local-controlled-preview";
import { useAgentLocalControlledTerminal } from "@/hooks/use-agent-local-controlled-terminal";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { useUnavailableModelFallback } from "@/hooks/use-unavailable-model-fallback";
import type { FileOperationGroups } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";

interface UseAgentLocalTabOpts {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  listFocused: boolean;
}

export function useAgentLocalTab({ navState, onSessionChange, onNavChange, listFocused }: UseAgentLocalTabOpts) {
  const { sessions, refresh, create, rename, remove, archive, updateModel, updateReasoning } = useAgentSessions();
  const projectsHook = useProjects();
  const activeSessionId = navState.sessionId;

  const activeSession = activeSessionId
    ? sessions.find((s) => s.id.localeCompare(activeSessionId) === 0)
    : null;
  const activeProject = activeSession?.project_id
    ? projectsHook.projects.find((p) => p.id === activeSession.project_id)
    : null;
  const terminalGroupKey = activeProject?.id || "__default__";
  const terminalCwd = activeProject?.path || "";
  const validGroupKeys = projectsHook.projects.map((p) => p.id);
  const terminalState = useTerminal(terminalGroupKey, terminalCwd, validGroupKeys);
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();
  const [welcomeModel, setWelcomeModel] = useState<{ model: string; provider: string } | null>(null);
  const [welcomeReasoningMode, setWelcomeReasoningMode] = useState<string | null>(null);
  const [fileOperations, setFileOperations] = useState<FileOperationGroups>({ all: [], latest: [] });

  const { groups: availableModels } = useAvailableModels();

  const normalizeForSelectedModel = useCallback(
    (nextModel: string, nextProvider: string, currentMode: string | null | undefined) => {
      const entry = availableModels.get(nextProvider)?.find((m) => m.id === nextModel) ?? null;
      const options = reasoningModeOptions(entry);
      return {
        mode: normalizeReasoningMode(currentMode, options, entry?.default_reasoning_mode),
        supportsThinking: entry?.supports_thinking ?? false,
      };
    },
    [availableModels],
  );

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;
  const reasoningMode = activeSession
    ? activeSession.reasoning_mode ?? (activeSession.thinking_enabled ? "auto" : null)
    : welcomeReasoningMode;
  const welcomeReasoning = normalizeForSelectedModel(
    currentDefault.model,
    currentDefault.provider,
    welcomeReasoningMode,
  );
  const filePreviewState = useFilePreview(activeSessionId ?? null, fileOperations.all, activeProject?.path);

  useUnavailableModelFallback({
    availableModels,
    model,
    provider,
    reasoningMode,
    activeSessionId,
    updateModel,
    setWelcomeModel,
  });

  const sessionActions = useSessionActions({
    create,
    rename,
    defaultModel,
    defaultProvider,
    welcomeModel,
    setWelcomeModel,
    welcomeReasoningMode,
    welcomeSupportsThinking: welcomeReasoning.supportsThinking,
    projectsHook,
    onSessionChange,
  });

  const setReasoningMode = useCallback((mode: ReasoningMode) => {
    const currentSupportsThinking = normalizeForSelectedModel(model, provider, mode).supportsThinking;
    if (activeSessionId) {
      void updateReasoning(activeSessionId, mode, currentSupportsThinking);
      return;
    }
    setWelcomeReasoningMode(mode);
  }, [model, provider, normalizeForSelectedModel, activeSessionId, updateReasoning]);

  const updateModelWithReasoning = useCallback(
    async (id: string, nextModel: string, nextProvider: string) => {
      const nextReasoning = normalizeForSelectedModel(nextModel, nextProvider, reasoningMode);
      await updateModel(
        id,
        nextModel,
        nextProvider,
        nextReasoning.mode,
        nextReasoning.supportsThinking,
      );
    },
    [normalizeForSelectedModel, reasoningMode, updateModel],
  );

  const handleSelectById = useCallback((id: string) => {
    onSessionChange?.(id);
  }, [onSessionChange]);

  const filePreview = useAgentLocalControlledPreview({ navState, filePreviewState, onNavChange });
  const terminal = useAgentLocalControlledTerminal({ navState, terminalState, terminalCwd, onNavChange });

  useAgentLocalShortcuts({
    activeSessionId,
    terminalOpen: terminal.isOpen,
    terminalTabsCount: terminal.tabs.length,
    terminalCwd,
    onAddTerminalTab: terminal.addTab,
    onToggleTerminal: terminal.togglePanel,
    onTogglePreview: filePreview.toggleOpen,
  });

  useAgentLocalTabPanelSync({ navState, filePreview: filePreviewState, terminal: terminalState });

  const visibleSessionIds = useMemo(() => {
    const projectIdSet = new Set(projectsHook.projects.map((p) => p.id));
    const visibleSessions = sessions.filter((s) => !s.parent_session_id && !s.clone_parent_session_id);
    const byProject = projectsHook.projects.flatMap((p) =>
      visibleSessions.filter((s) => s.project_id === p.id).map((s) => s.id),
    );
    const orphans = visibleSessions
      .filter((s) => !s.project_id || !projectIdSet.has(s.project_id))
      .map((s) => s.id);
    return [...byProject, ...orphans];
  }, [sessions, projectsHook.projects]);

  useArrowNavigation({
    items: visibleSessionIds,
    selectedId: activeSessionId,
    onSelect: (id) => void handleSelectById(id),
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const handleDeleteProject = useCallback((projectId: string) => {
    const entries = terminalState.getGroupPtyEntries(projectId);
    for (const { id, token } of entries) {
      invoke("pty_kill", { id, token }).catch(() => {});
    }
    terminalState.removeGroup(projectId);
    void projectsHook.remove(projectId);
  }, [terminalState, projectsHook]);

  const handleDeleteSession = useCallback(async (id: string) => {
    await archive(id);
    onSessionChange?.(null);
  }, [archive, onSessionChange]);

  return {
    sessions, refresh, rename, remove, archive, updateModel: updateModelWithReasoning,
    projectsHook, terminal, activeSession, activeSessionId,
    model, provider, currentDefault, activeProject,
    filePreview, fileOperations, setFileOperations,
    reasoningMode, setReasoningMode, welcomeModel, setWelcomeModel,
    sessionActions, handleSelectById, handleDeleteProject, handleDeleteSession,
  };
}
