import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

let watcherLifecycle = Promise.resolve();

export function useGitWatcher(projectPath: string | undefined) {
  useEffect(() => {
    if (!projectPath) return;

    enqueueWatcherCommand("start_git_watcher", projectPath);
    return () => {
      enqueueWatcherCommand("stop_git_watcher", projectPath);
    };
  }, [projectPath]);
}

function enqueueWatcherCommand(command: "start_git_watcher" | "stop_git_watcher", path: string) {
  watcherLifecycle = watcherLifecycle
    .then(() => invoke(command, { path }))
    .then(() => undefined)
    .catch(() => undefined);
}
