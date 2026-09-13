import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { X } from "@/components/ui/icons";
import { getThemeColorScheme, RESOLVED_THEME_OPTIONS, type ResolvedTheme } from "@/lib/app-themes";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { UpdateOperationRow } from "./update-operation-row";
import { useUpdateOperations } from "./use-update-operations";
import "./update-progress-window.css";

const THEMES = new Set<ResolvedTheme>(RESOLVED_THEME_OPTIONS.map(({ id }) => id));

export function UpdateProgressWindow() {
  const { t } = useTranslation();
  const { operations, dismiss, retry } = useUpdateOperations();
  const contentRef = useRef<HTMLElement>(null);
  useUpdateWindowTheme();
  useWindowHeight(contentRef, operations.length);
  if (operations.length === 0) return null;
  const restarting = operations.some(({ kind, phase, status }) =>
    kind === "app-release" && phase === "restarting" && status === "running");

  return (
    <main ref={contentRef} className="upw-window relief elev-above" aria-label={t("updates.window.title")}>
      <header className="upw-title" data-tauri-drag-region>
        <span className="upw-title-text" data-tauri-drag-region>
          {operations.length === 1 ? t("updates.window.title") : t("updates.window.count", { count: operations.length })}
        </span>
        {!restarting && (
          <button type="button" className="icon-btn" aria-label={t("updates.window.close")}
            onClick={() => void getCurrentWindow().close()}>
            <X size="var(--icon-sm)" />
          </button>
        )}
      </header>
      <ul className="upw-lines">
        {operations.map((operation) => (
          <UpdateOperationRow
            key={operation.id}
            operation={operation}
            onDismiss={dismiss}
            onRetry={retry}
          />
        ))}
      </ul>
    </main>
  );
}

function useWindowHeight(ref: React.RefObject<HTMLElement | null>, count: number) {
  const previous = useRef(0);
  useEffect(() => {
    const content = ref.current;
    if (!content) return;
    const resize = () => {
      const height = Math.ceil(content.scrollHeight);
      if (height === previous.current) return;
      previous.current = height;
      void invoke("resize_update_progress_window", { height }).catch(() => {});
    };
    const observer = new ResizeObserver(resize);
    observer.observe(content);
    resize();
    return () => observer.disconnect();
  }, [count, ref]);
}

function useUpdateWindowTheme() {
  useEffect(() => {
    applyTheme(readInitialTheme());
    const unlisten = listen<unknown>("beaver-theme-changed", ({ payload }) => {
      const theme = validatedTheme(payload);
      if (theme) applyTheme(theme);
    });
    return () => cleanupTauriListener(unlisten);
  }, []);
}

function readInitialTheme(): { palette: ResolvedTheme; colorScheme: "light" | "dark" } {
  try {
    const choice = localStorage.getItem("clgo-theme");
    if (THEMES.has(choice as ResolvedTheme)) {
      const palette = choice as ResolvedTheme;
      return { palette, colorScheme: getThemeColorScheme(palette) };
    }
    const base = localStorage.getItem("clgo-theme-base");
    if (base === "light" || base === "dark") return { palette: base, colorScheme: base };
  } catch {
    // Le thème système est le repli fermé si le stockage secondaire est indisponible.
  }
  const colorScheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  return { palette: colorScheme, colorScheme };
}

function validatedTheme(value: unknown): { palette: ResolvedTheme; colorScheme: "light" | "dark" } | null {
  if (!value || typeof value !== "object") return null;
  const candidate = value as Record<string, unknown>;
  if (typeof candidate.palette !== "string" || !THEMES.has(candidate.palette as ResolvedTheme)) return null;
  const palette = candidate.palette as ResolvedTheme;
  const colorScheme = getThemeColorScheme(palette);
  return candidate.colorScheme === colorScheme ? { palette, colorScheme } : null;
}

function applyTheme({ palette, colorScheme }: { palette: ResolvedTheme; colorScheme: "light" | "dark" }) {
  document.documentElement.dataset.palette = palette;
  document.documentElement.dataset.theme = colorScheme;
  document.documentElement.style.colorScheme = colorScheme;
}
