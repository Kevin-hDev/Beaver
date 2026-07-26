import { useState, useCallback, useEffect, useMemo } from "react";

export const FONT_SIZE_MIN = 10;
export const FONT_SIZE_MAX = 24;
const FONT_SIZE_DEFAULT = 18;
export type FontSize = number;

const LEGACY_FONT_SIZE_PX: Record<string, FontSize> = {
  "100": 18,
  "112": 20,
  "125": 22,
  "137": 24,
  "150": 24,
};

export const FONT_FAMILIES = [
  { id: "system", label: "System Default", value: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif' },
  { id: "jetbrains", label: "JetBrains Mono", value: '"JetBrains Mono", "Fira Code", monospace' },
  { id: "helvetica", label: "Helvetica Neue", value: '"Helvetica Neue", Helvetica, Arial, sans-serif' },
  { id: "menlo", label: "Menlo", value: 'Menlo, "SF Mono", Consolas, monospace' },
  { id: "ui-mono", label: "UI Monospace", value: 'ui-monospace, "SF Mono", "Cascadia Code", monospace' },
  { id: "pacifico", label: "Pacifico", value: '"Pacifico", cursive' },
  { id: "rancho", label: "Rancho", value: '"Rancho", cursive' },
] as const;

export type FontFamilyId = (typeof FONT_FAMILIES)[number]["id"];

/**
 * Thèmes de coloration de code disponibles dans /settings.
 * Chaque thème possède une variante dark + light (sélectionnée
 * automatiquement selon le mode app via l'attribut data-theme).
 */
export const CODE_THEMES = [
  { id: "default", label: "Défaut" },
  { id: "github", label: "GitHub" },
  { id: "one-dark-pro", label: "One Dark Pro" },
  { id: "tokyo-night", label: "Tokyo Night" },
  { id: "catppuccin", label: "Catppuccin" },
] as const;

export type CodeThemeId = (typeof CODE_THEMES)[number]["id"];

export function clampFontSizePx(value: number): FontSize {
  if (!Number.isFinite(value)) return FONT_SIZE_DEFAULT;
  return Math.min(FONT_SIZE_MAX, Math.max(FONT_SIZE_MIN, Math.round(value)));
}

export function parseStoredFontSize(raw: string | null): FontSize {
  const value = raw?.trim();
  if (!value) return FONT_SIZE_DEFAULT;
  if (value in LEGACY_FONT_SIZE_PX) return LEGACY_FONT_SIZE_PX[value];
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return FONT_SIZE_DEFAULT;
  return clampFontSizePx(parsed);
}

function loadFontSize(): FontSize {
  return parseStoredFontSize(localStorage.getItem("clgo-font-size"));
}

function loadFontFamily(): FontFamilyId {
  const saved = localStorage.getItem("clgo-font-family");
  if (saved && FONT_FAMILIES.some((f) => f.id === saved)) return saved as FontFamilyId;
  return "system";
}

function loadCodeTheme(): CodeThemeId {
  const saved = localStorage.getItem("clgo-code-theme");
  if (saved && CODE_THEMES.some((c) => c.id === saved)) return saved as CodeThemeId;
  return "default";
}

function loadSidebarExpand(): boolean {
  const saved = localStorage.getItem("clgo-sidebar-expand");
  return saved === null ? true : saved === "true";
}

function applyFontSize(fontSize: FontSize) {
  const next = clampFontSizePx(fontSize);
  document.documentElement.style.fontSize = `${next}px`;
  localStorage.setItem("clgo-font-size", String(next));
}

function applyFontFamily(fontFamilyId: FontFamilyId) {
  const fontFamily = FONT_FAMILIES.find((f) => f.id === fontFamilyId)!;
  document.documentElement.style.setProperty("--font-sans", fontFamily.value);
  localStorage.setItem("clgo-font-family", fontFamilyId);
}

function applyCodeTheme(codeThemeId: CodeThemeId) {
  document.documentElement.setAttribute("data-code-theme", codeThemeId);
  localStorage.setItem("clgo-code-theme", codeThemeId);
}

export function applyStoredSettings() {
  applyFontSize(loadFontSize());
  applyFontFamily(loadFontFamily());
  applyCodeTheme(loadCodeTheme());
}

export function useSettings() {
  const [fontSize, setFontSizeState] = useState<FontSize>(loadFontSize);
  const [fontFamilyId, setFontFamilyIdState] = useState<FontFamilyId>(loadFontFamily);
  const [codeThemeId, setCodeThemeIdState] = useState<CodeThemeId>(loadCodeTheme);
  const [sidebarExpand, setSidebarExpandState] = useState(loadSidebarExpand);

  const fontFamily = FONT_FAMILIES.find((f) => f.id === fontFamilyId)!;

  useEffect(() => {
    applyFontSize(fontSize);
  }, [fontSize]);

  useEffect(() => {
    applyFontFamily(fontFamilyId);
  }, [fontFamilyId]);

  useEffect(() => {
    applyCodeTheme(codeThemeId);
  }, [codeThemeId]);

  const setFontSize = useCallback((size: FontSize) => setFontSizeState(clampFontSizePx(size)), []);
  const setFontFamily = useCallback((id: FontFamilyId) => setFontFamilyIdState(id), []);
  const setCodeTheme = useCallback((id: CodeThemeId) => setCodeThemeIdState(id), []);
  const setSidebarExpand = useCallback((v: boolean) => {
    setSidebarExpandState(v);
    localStorage.setItem("clgo-sidebar-expand", String(v));
    window.dispatchEvent(new CustomEvent("clgo-sidebar-expand", { detail: v }));
  }, []);

  const decreaseFont = useCallback(() => {
    setFontSizeState((cur) => clampFontSizePx(cur - 1));
  }, []);

  const increaseFont = useCallback(() => {
    setFontSizeState((cur) => clampFontSizePx(cur + 1));
  }, []);

  return useMemo(() => ({
    fontSize, setFontSize, decreaseFont, increaseFont,
    fontFamilyId, fontFamily, setFontFamily,
    codeThemeId, setCodeTheme,
    sidebarExpand, setSidebarExpand,
  }) as const, [
    fontSize,
    setFontSize,
    decreaseFont,
    increaseFont,
    fontFamilyId,
    fontFamily,
    setFontFamily,
    codeThemeId,
    setCodeTheme,
    sidebarExpand,
    setSidebarExpand,
  ]);
}
