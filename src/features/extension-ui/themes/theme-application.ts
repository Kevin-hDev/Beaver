import {
  getThemeColorScheme,
  resolveTheme,
  type CoreThemeChoice,
  type ExtensionThemeChoice,
  type ResolvedTheme,
  type ThemeChoice,
} from "@/lib/app-themes";
import { UI_THEME_TOKENS } from "@/types/extension-ui-contract.generated";
import type { ExtensionThemeCatalog, ExtensionThemeEntry } from "./theme-catalog";

export type AppliedTheme = ResolvedTheme | ExtensionThemeChoice;

export function applyThemeChoice(
  target: HTMLElement,
  choice: ThemeChoice,
  catalog: ExtensionThemeCatalog,
  prefersDark: boolean,
): AppliedTheme | null {
  if (choice.startsWith("extension:")) {
    const entry = catalog.byChoice.get(choice as ExtensionThemeChoice);
    if (!entry) return null;
    applyExtensionTheme(target, entry);
    return entry.choice;
  }
  const resolved = resolveTheme(choice as CoreThemeChoice, prefersDark);
  applyCoreTheme(target, resolved);
  return resolved;
}

export function applyCoreTheme(target: HTMLElement, theme: ResolvedTheme): void {
  clearPublicThemeTokens(target);
  target.setAttribute("data-theme", getThemeColorScheme(theme));
  target.setAttribute("data-palette", theme);
}

export function applyExtensionTheme(target: HTMLElement, entry: ExtensionThemeEntry): void {
  clearPublicThemeTokens(target);
  target.setAttribute("data-theme", entry.colorScheme);
  target.setAttribute("data-palette", entry.paletteId);
  for (const [name, value] of Object.entries(entry.tokens)) {
    target.style.setProperty(name, value);
  }
}

export function clearPublicThemeTokens(target: HTMLElement): void {
  for (const token of UI_THEME_TOKENS) target.style.removeProperty(token);
}
