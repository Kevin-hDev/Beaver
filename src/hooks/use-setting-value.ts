import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

const EVENT_NAME = "clgo-advanced-settings-changed";

export function notifySettingsChanged() {
  window.dispatchEvent(new Event(EVENT_NAME));
}

export function useSettingValue<T>(key: string, fallback: T): T {
  const [value, setValue] = useState<T>(fallback);

  const load = useCallback(() => {
    invoke<Record<string, unknown>>("get_advanced_settings")
      .then((s) => {
        if (key in s) setValue(s[key] as T);
      })
      .catch(() => {});
  }, [key]);

  useEffect(() => {
    load();
    window.addEventListener(EVENT_NAME, load);
    return () => window.removeEventListener(EVENT_NAME, load);
  }, [load]);

  return value;
}
