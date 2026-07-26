import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AgentSourceSelection,
  AgentSourceSummary,
  SaveAgentSourceResult,
} from "@/types/agent-import";

export function useAgentImport() {
  const [sources, setSources] = useState<AgentSourceSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [failed, setFailed] = useState(false);

  const scan = useCallback(async () => {
    setLoading(true);
    setFailed(false);
    try {
      const result = await invoke<AgentSourceSummary[]>("scan_external_agent_sources");
      setSources(result);
    } catch {
      setSources([]);
      setFailed(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    invoke<AgentSourceSummary[]>("scan_external_agent_sources")
      .then((result) => {
        if (active) setSources(result);
      })
      .catch(() => {
        if (active) {
          setSources([]);
          setFailed(true);
        }
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const save = useCallback(async (
    selection: AgentSourceSelection,
    replaceDocuments = false,
  ) => {
    setSaving(true);
    try {
      const result = await invoke<SaveAgentSourceResult>(
        "save_external_agent_source_selection",
        { selection, replaceDocuments },
      );
      if (result.saved) await scan();
      return result;
    } finally {
      setSaving(false);
    }
  }, [scan]);

  return { sources, loading, saving, failed, scan, save };
}
