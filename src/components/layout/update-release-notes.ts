export type ReleaseNotesByLocale = Partial<Record<string, string[]>>;

const FALLBACK_LOCALE = "en";
const MAX_ITEMS = 6;
const MAX_TEXT_LENGTH = 180;

export function selectReleaseNotes(
  notesByLocale?: ReleaseNotesByLocale | null,
  language = FALLBACK_LOCALE,
): string[] {
  if (!notesByLocale) return [];

  const locale = normalizeLocale(language);
  const items =
    notesByLocale[locale] ??
    notesByLocale[locale.split("-")[0]] ??
    notesByLocale[FALLBACK_LOCALE] ??
    [];

  return items
    .filter((item): item is string => typeof item === "string")
    .filter((item) => item.trim() === item && item.length > 0)
    .filter((item) => [...item].length <= MAX_TEXT_LENGTH)
    .slice(0, MAX_ITEMS);
}

function normalizeLocale(language: string): string {
  return language.trim().toLowerCase().replace(/_/g, "-") || FALLBACK_LOCALE;
}
