import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  getMascotState,
  normalizeMascotState,
  saveMascotPosition,
  type MascotRuntimeAnimation,
} from "@/services/mascot";
import { MascotSprite } from "./mascot-sprite";
import { selectMascotAnimation } from "./use-mascot-animation";
import { useMascotDrag } from "./use-mascot-drag";
import "./mascot-overlay.css";

const STATE_EVENT = "mascot-state-changed";
const POSITION_SAVE_DELAY_MS = 250;

export function MascotOverlay() {
  const { t } = useTranslation();
  const [runtimeAnimation, setRuntimeAnimation] = useState<MascotRuntimeAnimation>("idle");
  const revision = useRef(0);
  const drag = useMascotDrag();
  const presentedAnimation = selectMascotAnimation(runtimeAnimation, drag.interactionAnimation);

  useEffect(() => {
    document.documentElement.classList.add("mco-page");
    void getCurrentWindow().setCursorIcon("grab").catch(() => {});
    return () => document.documentElement.classList.remove("mco-page");
  }, []);

  useEffect(() => {
    const applyState = (value: unknown) => {
      const next = normalizeMascotState(value);
      if (next.revision < revision.current) return;
      revision.current = next.revision;
      setRuntimeAnimation(next.animation);
    };
    void getMascotState().then(applyState).catch(() => {});
    const unlisten = listen<unknown>(STATE_EVENT, (event) => applyState(event.payload));
    return () => cleanupTauriListener(unlisten);
  }, []);

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    let saveTimer: number | undefined;
    const unlisten = currentWindow.onMoved((event) => {
      if (saveTimer !== undefined) window.clearTimeout(saveTimer);
      saveTimer = window.setTimeout(() => {
        void saveMascotPosition(event.payload.x, event.payload.y).catch(() => {});
      }, POSITION_SAVE_DELAY_MS);
    });
    return () => {
      if (saveTimer !== undefined) window.clearTimeout(saveTimer);
      cleanupTauriListener(unlisten);
    };
  }, []);

  return (
    <div
      className="mco-root"
      role="img"
      aria-label={t("settings.mascot.moveLabel")}
      onLostPointerCapture={drag.onLostPointerCapture}
      onPointerCancel={drag.onPointerCancel}
      onPointerDown={drag.onPointerDown}
      onPointerMove={drag.onPointerMove}
      onPointerUp={drag.onPointerUp}
      onContextMenu={(event) => event.preventDefault()}
    >
      <MascotSprite animation={presentedAnimation} active width="100%" />
    </div>
  );
}
