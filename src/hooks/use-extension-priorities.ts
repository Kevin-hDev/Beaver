import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseExtensionDiscoveryPreferences } from "@/lib/extension-discovery";
import { extensionErrorKey } from "@/lib/extension-errors";

export function useExtensionPriorities(
  setOperationError: (value: string | null) => void,
) {
  const [protectedPluginIds, setProtectedPluginIds] = useState<string[]>([]);
  const [priorityBusy, setPriorityBusy] = useState(false);

  const applyValue = useCallback((value: unknown) => {
    setProtectedPluginIds(
      parseExtensionDiscoveryPreferences(value).protectedPluginIds,
    );
  }, []);
  const setPriorityPlugins = useCallback(async (pluginIds: string[]) => {
    setOperationError(null);
    setPriorityBusy(true);
    try {
      const value = await invoke<unknown>("set_extension_discovery_preferences", {
        protectedPluginIds: pluginIds,
      });
      applyValue(value);
      return true;
    } catch (error) {
      setOperationError(extensionErrorKey(error, "extensions.errors.operation"));
      return false;
    } finally {
      setPriorityBusy(false);
    }
  }, [applyValue, setOperationError]);

  return {
    protectedPluginIds,
    priorityBusy,
    applyValue,
    setPriorityPlugins,
  };
}
