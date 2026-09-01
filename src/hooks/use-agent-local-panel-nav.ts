import { useEffect, useRef } from "react";
import type { useFileTree } from "@/hooks/use-file-tree";
import type { AgentLocalNavState } from "@/types/navigation";

interface AgentLocalPanelNavArgs {
  navState: AgentLocalNavState;
  fileTree: ReturnType<typeof useFileTree>;
}

export function useAgentLocalPanelNav({
  navState,
  fileTree,
}: AgentLocalPanelNavArgs) {
  const restoredNavKey = useRef<string | null>(null);
  const { open: fileTreeOpen, setOpen: setFileTreeOpen } = fileTree;
  const navKey = JSON.stringify([
    navState.sessionId,
    navState.fileTreeOpen,
  ]);

  useEffect(() => {
    if (restoredNavKey.current === navKey) return;
    restoredNavKey.current = navKey;
    if (fileTreeOpen !== navState.fileTreeOpen) {
      setFileTreeOpen(navState.fileTreeOpen);
    }
  }, [
    navKey, fileTreeOpen, setFileTreeOpen, navState.fileTreeOpen, navState.sessionId,
  ]);
}
