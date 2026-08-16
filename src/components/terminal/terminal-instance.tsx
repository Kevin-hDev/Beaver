import { useEffect, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { IS_MAC } from "@/lib/platform";
import i18n from "@/i18n";
import { readTerminalTheme, readTerminalFont } from "./terminal-theme";
import "@xterm/xterm/css/xterm.css";

interface TerminalInstanceProps {
  tabId: string;
  cwd: string;
  isVisible: boolean;
  onPtyReady: (tabId: string, ptyId: number, ptyToken: string) => void;
  onExit: (tabId: string) => void;
  onActivity: (tabId: string, hasActivity: boolean) => void;
  onTogglePanel?: () => void;
}

export function TerminalInstance({
  tabId,
  cwd,
  isVisible,
  onPtyReady,
  onExit,
  onActivity,
  onTogglePanel,
}: TerminalInstanceProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const ptyIdRef = useRef<number | null>(null);
  const ptyTokenRef = useRef<string | null>(null);
  /* Le flux est branché une fois pour toutes au montage : il ne verrait qu'un
     état figé de la visibilité et du rappel s'ils y étaient lus directement. */
  const visibleRef = useRef(isVisible);
  const activityRef = useRef(onActivity);
  useEffect(() => {
    visibleRef.current = isVisible;
    activityRef.current = onActivity;
  });

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      theme: readTerminalTheme(),
      fontFamily: readTerminalFont(),
      fontSize: 13,
      cursorBlink: true,
      cursorStyle: "bar",
      cursorWidth: 2,
      allowProposedApi: true,
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
        onTogglePanel?.();
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

    const pasteHandler = (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData("text");
      if (text && ptyIdRef.current !== null && ptyTokenRef.current) {
        invoke("pty_write", { id: ptyIdRef.current, token: ptyTokenRef.current, data: text }).catch(() => {});
        e.preventDefault();
      }
    };
    containerRef.current.addEventListener("paste", pasteHandler);

    let disposed = false;

    const channel = new Channel<{ data: string; isExit: boolean; exitCode: number }>();
    channel.onmessage = (event) => {
      if (disposed) return;
      if (event.isExit) {
        term.writeln(`\r\n[${i18n.t("terminal.processExited", { code: event.exitCode })}]`);
        ptyIdRef.current = null;
        onExit(tabId);
      } else {
        term.write(event.data);
        /* Hors des regards, le texte qui arrive vaut une marque sur l'onglet. */
        if (!visibleRef.current) activityRef.current(tabId, true);
      }
    };

    invoke<{ id: number; token: string }>("pty_spawn", {
      cwd: cwd || null,
      cols: term.cols || 80,
      rows: term.rows || 24,
      onOutput: channel,
    }).then(({ id, token }) => {
      if (disposed) {
        invoke("pty_kill", { id, token }).catch(() => {});
        return;
      }
      ptyIdRef.current = id;
      ptyTokenRef.current = token;
      onPtyReady(tabId, id, token);

      term.onData((data) => {
        invoke("pty_write", { id, token, data }).catch(() => {});
      });

      term.onResize(({ cols, rows }) => {
        invoke("pty_resize", { id, token, cols, rows }).catch(() => {});
      });
    }).catch(() => {
      if (!disposed) {
        term.writeln(`\r\n${i18n.t("terminal.failedToStart")}\r\n`);
      }
    });

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

    const container = containerRef.current;
    return () => {
      disposed = true;
      clearTimeout(resizeTimer);
      container?.removeEventListener("paste", pasteHandler);
      resizeObserver.disconnect();
      if (ptyIdRef.current !== null && ptyTokenRef.current) {
        invoke("pty_kill", { id: ptyIdRef.current, token: ptyTokenRef.current }).catch(() => {});
      }
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
    const observer = new MutationObserver(() => {
      if (termRef.current) {
        termRef.current.options.theme = readTerminalTheme();
      }
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => observer.disconnect();
  }, []);

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
