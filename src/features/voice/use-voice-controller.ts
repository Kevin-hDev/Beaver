import { useCallback, useEffect, useMemo, useState } from "react";
import { IS_LINUX } from "@/lib/platform";
import { matchesAppShortcut } from "@/lib/app-shortcuts";
import { matchesVoiceShortcut } from "./voice-keyboard";
import type { VoiceLanguage, VoiceSettings } from "@/types/voice.generated";
import { dispatchVoiceAction, getVoiceCatalog, getVoiceSettings, listVoiceDevices, updateVoiceSettings } from "./voice-client";
import { acceptVoiceSnapshot, useVoiceSnapshot } from "./voice-store";

let contextGeneration = 0;

export function useVoiceController(draftKey: string) {
  const snapshot = useVoiceSnapshot();
  const [settings, setSettings] = useState<VoiceSettings | null>(null);
  const [deviceCount, setDeviceCount] = useState<number | null>(null);
  const [dialog, setDialog] = useState<"first-use" | "language" | null>(null);
  const [pending, setPending] = useState(false);
  const destination = snapshot?.operation?.destination;
  const origin = destination?.kind === "draft" && destination.draft_key === draftKey;

  const refreshDevices = useCallback(async () => {
    if (IS_LINUX) return;
    try { setDeviceCount((await listVoiceDevices()).length); } catch { setDeviceCount(0); }
  }, []);

  useEffect(() => {
    if (IS_LINUX) return;
    void getVoiceSettings().then(setSettings).catch(() => setSettings(null));
    void Promise.resolve().then(refreshDevices);
    const onFocus = () => { void refreshDevices(); };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [refreshDevices]);

  const run = useCallback(async (action: Parameters<typeof dispatchVoiceAction>[0]) => {
    if (pending) return;
    setPending(true);
    try { acceptVoiceSnapshot(await dispatchVoiceAction(action)); } finally { setPending(false); }
  }, [pending]);

  const begin = useCallback(() => {
    if (!settings?.explanation_accepted) setDialog("first-use");
    else setDialog("language");
  }, [settings]);

  const acceptExplanation = useCallback(async () => {
    const next = await updateVoiceSettings({ explanation_accepted: true });
    setSettings(next);
    const selected = (await getVoiceCatalog()).find((item) => item.model === next.model);
    if (selected && !selected.installed) {
      acceptVoiceSnapshot(await dispatchVoiceAction({ action: "install", model_id: selected.id }));
      setDialog(null);
    } else {
      setDialog("language");
    }
  }, []);

  const start = useCallback(async (language: VoiceLanguage | null) => {
    setDialog(null);
    contextGeneration = (contextGeneration + 1) % Number.MAX_SAFE_INTEGER || 1;
    await run({ action: "start", destination: { kind: "draft", draft_key: draftKey }, context_generation: contextGeneration, language });
  }, [draftKey, run]);

  const operationId = origin ? snapshot?.operation?.id : null;
  useEffect(() => {
    if (IS_LINUX) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || !(settings?.shortcut ? matchesVoiceShortcut(event, settings.shortcut) : matchesAppShortcut(event, "toggleVoice"))) return;
      event.preventDefault();
      const activeId = snapshot?.operation?.id;
      if (snapshot?.phase === "listening" && activeId) void run({ action: "validate", operation_id: activeId });
      else if (snapshot?.phase === "idle") begin();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [begin, run, settings?.shortcut, snapshot]);
  return useMemo(() => ({
    snapshot, settings, origin, activeElsewhere: Boolean(snapshot?.operation && !origin),
    available: !IS_LINUX && settings?.enabled !== false && deviceCount !== 0,
    checkingDevices: deviceCount === null, dialog, pending, begin, acceptExplanation,
    closeDialog: () => setDialog(null), start, refreshDevices,
    validate: () => operationId && run({ action: "validate", operation_id: operationId }),
    cancel: () => operationId && run({ action: "cancel-insertion", operation_id: operationId }),
  }), [snapshot, settings, origin, deviceCount, dialog, pending, begin, acceptExplanation, start, refreshDevices, operationId, run]);
}
