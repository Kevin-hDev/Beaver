import { useCallback, useEffect, useMemo, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { normalizeFileOperationPath } from "@/lib/file-preview-utils";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { checkPreviewFilesExist } from "@/services/file-preview";
import type { FileOperation } from "@/types/file-preview";

export function usePreviewFallbackExistence(
  fallbackOps: FileOperation[],
  baseDir: string | undefined,
  onMissing: (missingKeys: Set<string>) => void,
) {
  const requestRef = useRef(0);
  const paths = useMemo(() => fallbackOps.map((operation) => operation.path), [fallbackOps]);

  const refreshExists = useCallback(() => {
    const requestId = ++requestRef.current;
    if (paths.length === 0) return;
    checkPreviewFilesExist(paths, baseDir)
      .then((results) => {
        if (requestId !== requestRef.current) return;
        const missingKeys = new Set(
          results
            .filter((result) => !result.exists)
            .map((result) => normalizeFileOperationPath(result.path)),
        );
        if (missingKeys.size > 0) onMissing(missingKeys);
      })
      .catch(() => {});
  }, [paths, baseDir, onMissing]);

  useEffect(() => {
    refreshExists();
    return () => { requestRef.current += 1; };
  }, [refreshExists]);

  useEffect(() => {
    const unlisten = listen("file-tree-changed", () => refreshExists());
    return () => { cleanupTauriListener(unlisten); };
  }, [refreshExists]);
}
