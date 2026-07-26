import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  DEFAULT_MASCOT_SETTINGS,
  getMascotSettings,
  normalizeMascotSettings,
  patchMascotSettings,
  type MascotSettings,
  type MascotSettingsPatch,
} from "@/services/mascot";

const SETTINGS_EVENT = "mascot-settings-changed";

export function useMascotSettings() {
  const [settings, setSettings] = useState<MascotSettings>(DEFAULT_MASCOT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const requestId = useRef(0);

  const refresh = useCallback(async () => {
    const id = ++requestId.current;
    try {
      const next = await getMascotSettings();
      if (id === requestId.current) setSettings(next);
    } finally {
      if (id === requestId.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh().catch(() => {});
    const unlisten = listen<unknown>(SETTINGS_EVENT, (event) => {
      requestId.current += 1;
      setSettings(normalizeMascotSettings(event.payload));
      setLoading(false);
    });
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);

  const update = useCallback(async (patch: MascotSettingsPatch) => {
    const id = ++requestId.current;
    const previous = settings;
    const optimistic = normalizeMascotSettings({ ...settings, ...patch });
    setSettings(optimistic);
    try {
      const saved = await patchMascotSettings(patch);
      if (id === requestId.current) setSettings(saved);
      return saved;
    } catch (error) {
      if (id === requestId.current) setSettings(previous);
      throw error;
    }
  }, [settings]);

  return { settings, loading, update, refresh };
}
