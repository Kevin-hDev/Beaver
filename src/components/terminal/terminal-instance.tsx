import { useEffect, useRef } from "react";
import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { IS_MAC } from "@/lib/platform";
import { createTerminalPtyBridge } from "./terminal-pty-bridge";
import { readTerminalFont } from "./terminal-theme";
import "@xterm/xterm/css/xterm.css";

interface TerminalInstanceProps {
  tabId: string;
  groupKey: string;
  theme: ITheme;
  isVisible: boolean;
  onPtyReady: (tabId: string, ptyId: number, ptyToken: string) => void;
  onExit: (tabId: string) => void;
  onActivity: (tabId: string, hasActivity: boolean) => void;
  onTogglePanel?: () => void;
}

export function TerminalInstance({
  tabId,
  groupKey,
  theme,
  isVisible,
  onPtyReady,
  onExit,
  onActivity,
  onTogglePanel,
}: TerminalInstanceProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  /* Le flux est branché une fois pour toutes au montage : il ne verrait qu'un
     état figé de la visibilité et du rappel s'ils y étaient lus directement. */
  const visibleRef = useRef(isVisible);
  const activityRef = useRef(onActivity);
  const readyRef = useRef(onPtyReady);
  const exitRef = useRef(onExit);
  const toggleRef = useRef(onTogglePanel);
  useEffect(() => {
    visibleRef.current = isVisible;
    activityRef.current = onActivity;
    readyRef.current = onPtyReady;
    exitRef.current = onExit;
    toggleRef.current = onTogglePanel;
  });

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      theme,
      fontFamily: readTerminalFont(),
      fontSize: 13,
      cursorBlink: true,
      cursorStyle: "bar",
      cursorWidth: 2,
      rightClickSelectsWord: true,
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);

    fit.fit();
    termRef.current = term;
    fitRef.current = fit;

    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== "keydown") return true;

      const toggleMod = IS_MAC ? e.metaKey : e.ctrlKey;
      if (toggleMod && e.code === "KeyJ") {
        toggleRef.current?.();
        return false;
      }

      const copyPasteMod = IS_MAC ? e.metaKey : e.ctrlKey;

      if (copyPasteMod && e.code === "KeyC") {
        const selection = term.getSelection();
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {});
          return false;
        }
        return true;
      }

      if (copyPasteMod && e.code === "KeyV") {
        return true;
      }

      return true;
    });

    const bridge = createTerminalPtyBridge({
      tabId,
      groupKey,
      terminal: term,
      isVisible: () => visibleRef.current,
      onPtyReady: (id, ptyId, token) => readyRef.current(id, ptyId, token),
      onExit: (id) => exitRef.current(id),
      onActivity: (id, hasActivity) => activityRef.current(id, hasActivity),
    });
    void bridge.start();
    const resizeSubscription = term.onResize(({ cols, rows }) => bridge.resize(cols, rows));

    let resizeTimer: ReturnType<typeof setTimeout>;
    const resizeObserver = new ResizeObserver(() => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        /* Replié, le panneau donne une hauteur nulle à ses écrans : ajuster
           là-dessus réduirait le terminal à une ligne et bousculerait ce qui y
           tourne. */
        const host = containerRef.current;
        if (host && host.offsetWidth > 0 && host.offsetHeight > 0) {
          fit.fit();
        }
      }, 100);
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      clearTimeout(resizeTimer);
      resizeObserver.disconnect();
      resizeSubscription.dispose();
      bridge.dispose();
      term.dispose();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- PTY mount-only setup
  }, []);

  useEffect(() => {
    if (!isVisible) return;
    /* Visible vaut vu : la marque tombe ici, et nulle part ailleurs. */
    onActivity(tabId, false);
    if (!fitRef.current) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        fitRef.current?.fit();
        termRef.current?.focus();
      });
    });
  }, [isVisible, tabId, onActivity]);

  useEffect(() => {
    if (termRef.current) termRef.current.options.theme = theme;
  }, [theme]);

  return (
    <div
      ref={containerRef}
      className="terminal-screen"
      data-keyboard-scope="local"
      /* Seul l'escamotage est dynamique : la forme de l'écran vit en CSS. */
      style={{
        visibility: isVisible ? "visible" : "hidden",
        position: isVisible ? "relative" : "absolute",
      }}
    />
  );
}
