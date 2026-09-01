import { useCallback, useMemo, useState } from "react";
import type { FileOperationGroups } from "@/types/file-preview";

const MAX_TRACKED_SESSIONS = 32;
const EMPTY: FileOperationGroups = { all: [], latest: [] };

interface OwnedOperations {
  ownerKey: string | null;
  groups: FileOperationGroups;
}

export function useOwnedFileOperations(ownerKey: string | null) {
  const [entries, setEntries] = useState<OwnedOperations[]>([]);
  const operations = useMemo(
    () => entries.find((entry) => entry.ownerKey === ownerKey)?.groups ?? EMPTY,
    [entries, ownerKey],
  );
  const setOperations = useCallback((groups: FileOperationGroups) => {
    setEntries((previous) => [
      ...previous.filter((entry) => entry.ownerKey !== ownerKey),
      { ownerKey, groups },
    ].slice(-MAX_TRACKED_SESSIONS));
  }, [ownerKey]);

  return { operations, setOperations };
}
