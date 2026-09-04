export const RESOLVED_THEME_OPTIONS = [
  {
    id: "light", labelKey: "settings.light", colorScheme: "light",
    cssPath: "src/styles/themes/light.css",
  },
  {
    id: "dark", labelKey: "settings.dark", colorScheme: "dark",
    cssPath: "src/styles/themes/dark.css",
  },
  {
    id: "emerald-night", labelKey: "settings.emeraldNight", colorScheme: "dark",
    cssPath: "src/styles/themes/emerald-night.css",
  },
  {
    id: "cobalt-frost", labelKey: "settings.cobaltFrost", colorScheme: "light",
    cssPath: "src/styles/themes/cobalt-frost.css",
  },
  {
    id: "astral-mist", labelKey: "settings.astralMist", colorScheme: "dark",
    cssPath: "src/styles/themes/astral-mist.css",
  },
  {
    id: "crimson-eclipse", labelKey: "settings.crimsonEclipse", colorScheme: "dark",
    cssPath: "src/styles/themes/crimson-eclipse.css",
  },
] as const;

export const THEME_OPTIONS = [
  ...RESOLVED_THEME_OPTIONS,
  { id: "system", labelKey: "settings.system", colorScheme: "system" },
] as const;

export type ResolvedTheme = (typeof RESOLVED_THEME_OPTIONS)[number]["id"];
export type CoreThemeChoice = (typeof THEME_OPTIONS)[number]["id"];
export type ExtensionThemeChoice = `extension:${string}`;
export type ThemeChoice = CoreThemeChoice | ExtensionThemeChoice;
export type ThemeColorScheme = "light" | "dark";

const COLOR_SCHEME_BY_THEME = Object.fromEntries(
  RESOLVED_THEME_OPTIONS.map(({ id, colorScheme }) => [id, colorScheme]),
) as Record<ResolvedTheme, ThemeColorScheme>;

export function isCoreThemeChoice(value: string | null): value is CoreThemeChoice {
  return THEME_OPTIONS.some((option) => option.id === value);
}

export function isThemeChoice(
  value: string | null,
  extensionChoices: readonly ExtensionThemeChoice[] = [],
): value is ThemeChoice {
  return isCoreThemeChoice(value) || extensionChoices.some((choice) => choice === value);
}

export function resolveTheme(choice: CoreThemeChoice, prefersDark: boolean): ResolvedTheme {
  if (choice !== "system") return choice;
  return prefersDark ? "dark" : "light";
}

export function getThemeColorScheme(theme: ResolvedTheme): ThemeColorScheme {
  return COLOR_SCHEME_BY_THEME[theme];
}

export function getNextThemeChoice(
  choice: ThemeChoice,
  choices: readonly ThemeChoice[] = THEME_OPTIONS.map(({ id }) => id),
): ThemeChoice {
  if (choices.length === 0) return "system";
  const currentIndex = choices.findIndex((option) => option === choice);
  return choices[(currentIndex + 1) % choices.length];
}
