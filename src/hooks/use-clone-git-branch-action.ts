import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { useGitBranch } from "@/hooks/use-git-branch";
import { showAppError } from "@/lib/app-error";
import type { SessionTab } from "@/types/agent";

type CloneGitBranchActionState = "idle" | "loading" | "success";

interface Options {
  projectPath?: string;
  git: Pick<ReturnType<typeof useGitBranch>, "isGitRepo" | "refresh">;
  isStreaming: boolean;
  activeSessionTab?: SessionTab | null;
  onCreateCloneGitBranch?: (path: string, cloneSessionId: string) => Promise<string>;
}

export function useCloneGitBranchAction({
  projectPath,
  git,
  isStreaming,
  activeSessionTab,
  onCreateCloneGitBranch,
}: Options) {
  const { t } = useTranslation();
  const [rawState, setRawState] = useState<{
    sessionId: string | null;
    status: "idle" | "loading" | "success";
  }>({ sessionId: null, status: "idle" });
  const state = rawState.sessionId === activeSessionTab?.session_id ? rawState.status : "idle";

  useEffect(() => {
    if (!rawState.sessionId || rawState.sessionId === activeSessionTab?.session_id) return;
    const timer = window.setTimeout(() => setRawState({ sessionId: null, status: "idle" }), 0);
    return () => window.clearTimeout(timer);
  }, [activeSessionTab?.session_id, rawState.sessionId]);

  useEffect(() => {
    if (rawState.status !== "success") return;
    const timer = window.setTimeout(() => {
      setRawState({ sessionId: null, status: "idle" });
    }, 3000);
    return () => window.clearTimeout(timer);
  }, [rawState.status]);

  const isCloneTab = !!activeSessionTab
    && !activeSessionTab.is_main
    && !!activeSessionTab.clone_parent_session_id;
  const eligible = !!projectPath
    && !!onCreateCloneGitBranch
    && isCloneTab
    && !activeSessionTab.git_branch
    && git.isGitRepo;
  // Une fois la branche liée, on masque le bouton V2 : le BranchSelector affiche
  // déjà la branche courante, l'utilisateur garde son switch/création manuels.
  // On ne re-vient qu'aux états transitoires loading/success (pendant puis juste
  // après la création), puis on disparaît.
  const visible = eligible || state === "loading" || state === "success";
  const effectiveState: CloneGitBranchActionState = state;
  const disabled = isStreaming || effectiveState !== "idle";

  const label = useMemo(() => {
    if (effectiveState === "loading") return t("agentLocal.clone.gitCreating");
    if (effectiveState === "success") return t("agentLocal.clone.gitCreated");
    return t("agentLocal.clone.gitCreate");
  }, [effectiveState, t]);

  const onCreate = useCallback(() => {
    if (disabled || !eligible || !activeSessionTab || !projectPath || !onCreateCloneGitBranch) {
      return;
    }
    void (async () => {
      setRawState({ sessionId: activeSessionTab.session_id, status: "loading" });
      try {
        await onCreateCloneGitBranch(projectPath, activeSessionTab.session_id);
        await git.refresh();
        setRawState({ sessionId: activeSessionTab.session_id, status: "success" });
      } catch (error) {
        showAppError(error, t, "branches.errorInternal");
        setRawState({ sessionId: activeSessionTab.session_id, status: "idle" });
      }
    })();
  }, [activeSessionTab, disabled, eligible, git, onCreateCloneGitBranch, projectPath, t]);

  return { visible, state: effectiveState, label, disabled, onCreate };
}
