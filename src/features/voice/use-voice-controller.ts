import { useCallback, useEffect, useMemo, useState } from "react";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import { IS_LINUX } from "@/lib/platform";
import { matchesAppShortcut } from "@/lib/app-shortcuts";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";
import { matchesVoiceShortcut } from "./voice-keyboard";
import { resolveVoiceLanguage } from "./voice-language-options";
import type { VoiceLanguage, VoiceLanguageMode, VoiceSettings } from "@/types/voice.generated";
import { dispatchVoiceAction, getVoiceCatalog, getVoiceSettings, listVoiceDevices, updateVoiceSettings } from "./voice-client";
import { acceptVoiceSnapshot, useVoiceSnapshot } from "./voice-store";

let contextGeneration = 0;

export function useVoiceController(draftKey: string) {
  const surfaceActive = useAppSurfaceActive();
  const snapshot = useVoiceSnapshot();
  const downloads = useModelDownloads();
  const cancelModelDownload = downloads.cancelDownload;
  const resumeModelDownload = downloads.resumeDownload;
  const [settings, setSettings] = useState<VoiceSettings | null>(null);
  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);
  const [languageMode, setLanguageMode] = useState<VoiceLanguageMode>("automatic-only");
  const [deviceCount, setDeviceCount] = useState<number | null>(null);
  const [dialog, setDialog] = useState<"first-use" | null>(null);
  const [pending, setPending] = useState(false);
  const destination = snapshot?.operation?.destination;
  const origin = destination?.kind === "draft" && destination.draft_key === draftKey;

  const refreshDevices = useCallback(async () => {
    if (IS_LINUX) return;
    try { setDeviceCount((await listVoiceDevices()).length); } catch { setDeviceCount(0); }
  }, []);

  useEffect(() => {
    if (IS_LINUX || !surfaceActive) return;
    void Promise.all([getVoiceSettings(), getVoiceCatalog()]).then(([next, catalog]) => {
      setSettings(next);
      const selected = catalog.find((item) => item.model === next.model);
      setSelectedModelId(selected?.id ?? null);
      setLanguageMode(selected?.languageMode ?? "automatic-only");
    }).catch(() => setSettings(null));
    void Promise.resolve().then(refreshDevices);
    const onFocus = () => { void refreshDevices(); };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [refreshDevices, surfaceActive]);

  const run = useCallback(async (action: Parameters<typeof dispatchVoiceAction>[0]) => {
    if (pending) return;
    setPending(true);
    try {
      acceptVoiceSnapshot(await dispatchVoiceAction(action));
    } catch {
      showToast(i18n.t("errors.operationFailed"), "error");
    } finally {
      setPending(false);
    }
  }, [pending]);

  const start = useCallback(async (savedLanguage: VoiceLanguage | undefined = settings?.language, selectedLanguageMode = languageMode) => {
    setDialog(null);
    contextGeneration = (contextGeneration + 1) % Number.MAX_SAFE_INTEGER || 1;
    const language = savedLanguage
      ? resolveVoiceLanguage(savedLanguage, i18n.resolvedLanguage ?? i18n.language, selectedLanguageMode)
      : null;
    await run({ action: "start", destination: { kind: "draft", draft_key: draftKey }, context_generation: contextGeneration, language });
  }, [draftKey, languageMode, run, settings?.language]);

  const begin = useCallback(() => {
    if (!settings?.explanation_accepted) setDialog("first-use");
    else void start();
  }, [settings?.explanation_accepted, start]);

  const acceptExplanation = useCallback(async () => {
    try {
      const next = await updateVoiceSettings({ explanation_accepted: true });
      setSettings(next);
      const selected = (await getVoiceCatalog()).find((item) => item.model === next.model);
      setSelectedModelId(selected?.id ?? null);
      setLanguageMode(selected?.languageMode ?? "automatic-only");
      if (selected && !selected.installed) {
        acceptVoiceSnapshot(await dispatchVoiceAction({ action: "install", model_id: selected.id }));
        setDialog(null);
      } else {
        await start(next.language, selected?.languageMode ?? "automatic-only");
      }
    } catch {
      showToast(i18n.t("errors.operationFailed"), "error");
    }
  }, [start]);

  const operationId = origin ? snapshot?.operation?.id : null;
  const modelDownload = downloads.downloads.find((item) => item.kind === "voice"
    && item.modelId === selectedModelId
    && ["queued", "running", "cancelling", "suspended"].includes(item.status)) ?? null;
  useEffect(() => {
    if (IS_LINUX || !surfaceActive) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || !(settings?.shortcut ? matchesVoiceShortcut(event, settings.shortcut) : matchesAppShortcut(event, "toggleVoice"))) return;
      event.preventDefault();
      const activeId = snapshot?.operation?.id;
      if (snapshot?.phase === "listening" && activeId) void run({ action: "validate", operation_id: activeId });
      else if (snapshot?.phase === "idle") begin();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [begin, run, settings?.shortcut, snapshot, surfaceActive]);
  return useMemo(() => ({
    snapshot, settings, origin, activeElsewhere: Boolean(snapshot?.operation && !origin),
    available: !IS_LINUX && settings?.enabled !== false && deviceCount !== 0,
    checkingDevices: deviceCount === null, dialog, pending, begin, acceptExplanation,
    closeDialog: () => setDialog(null), start, refreshDevices,
    modelDownload,
    cancelDownload: (id: string) => cancelModelDownload(id).catch(() => showToast(i18n.t("errors.operationFailed"), "error")),
    resumeDownload: (id: string) => resumeModelDownload(id).catch(() => showToast(i18n.t("errors.operationFailed"), "error")),
    validate: () => operationId && run({ action: "validate", operation_id: operationId }),
    cancel: () => operationId && run({ action: "cancel-insertion", operation_id: operationId }),
  }), [snapshot, settings, origin, deviceCount, dialog, pending, begin, acceptExplanation, start, refreshDevices, modelDownload, cancelModelDownload, resumeModelDownload, operationId, run]);
}
