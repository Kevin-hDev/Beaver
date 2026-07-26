import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { FilePreviewActiveTab } from "@/types/file-preview";

export function usePrunePreviewTabs(
  operationById: ReadonlyMap<string, unknown>,
  setTabIds: Dispatch<SetStateAction<string[]>>,
  setActiveTab: Dispatch<SetStateAction<FilePreviewActiveTab>>,
) {
  useEffect(() => {
    setTabIds((ids) => {
      const next = ids.filter((id) => operationById.has(id));
      return next.length === ids.length ? ids : next;
    });
    setActiveTab((current) => (
      current === "summary" || operationById.has(current) ? current : "summary"
    ));
  }, [operationById, setActiveTab, setTabIds]);
}
