import {
  RESOLVED_THEME_OPTIONS,
  type ExtensionThemeChoice,
  type ThemeColorScheme,
} from "@/lib/app-themes";
import { localizedText } from "../standard/localized-text";
import type {
  StandardCatalogSnapshot,
  StandardThemeContribution,
} from "../standard/types";
import {
  extensionThemeChoice,
  parseExtensionThemeTokens,
} from "./theme-parser";

export interface ExtensionThemeEntry {
  choice: ExtensionThemeChoice;
  paletteId: string;
  extensionId: string;
  sourceName: string;
  label: string;
  colorScheme: ThemeColorScheme;
  tokens: Readonly<Record<string, string>>;
}

export interface ExtensionThemeCatalog {
  ready: boolean;
  entries: readonly ExtensionThemeEntry[];
  byChoice: ReadonlyMap<ExtensionThemeChoice, ExtensionThemeEntry>;
}

export const PENDING_THEME_CATALOG: ExtensionThemeCatalog = {
  ready: false,
  entries: [],
  byChoice: new Map(),
};

export function buildThemeCatalog(
  snapshot: StandardCatalogSnapshot | null,
  extensionNames: ReadonlyMap<string, string>,
  locale: string,
  ready: boolean,
): ExtensionThemeCatalog {
  if (!ready) return PENDING_THEME_CATALOG;
  const entries = (snapshot?.contributions ?? []).flatMap((entry) => {
    if (entry.contribution.type !== "theme") return [];
    return [themeEntry(
      entry.extensionId,
      entry.contribution,
      extensionNames.get(entry.extensionId) ?? entry.extensionId,
      locale,
    )];
  });
  entries.sort((left, right) => left.label.localeCompare(right.label, locale)
    || left.paletteId.localeCompare(right.paletteId));
  return {
    ready: true,
    entries,
    byChoice: new Map(entries.map((entry) => [entry.choice, entry])),
  };
}

export function coreThemeCssPaths(): readonly string[] {
  return RESOLVED_THEME_OPTIONS.map(({ cssPath }) => cssPath);
}

function themeEntry(
  extensionId: string,
  theme: StandardThemeContribution,
  sourceName: string,
  locale: string,
): ExtensionThemeEntry {
  return {
    choice: extensionThemeChoice(theme.id),
    paletteId: theme.id,
    extensionId,
    sourceName,
    label: localizedText(theme.label, locale),
    colorScheme: theme.base,
    tokens: parseExtensionThemeTokens(theme.tokens),
  };
}
