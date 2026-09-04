import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/components/ui/toast";
import {
  THEME_OPTIONS,
  getNextThemeChoice,
  getThemeColorScheme,
  isCoreThemeChoice,
  isThemeChoice,
  resolveTheme,
  type ExtensionThemeChoice,
  type ResolvedTheme,
  type ThemeChoice,
} from "@/lib/app-themes";
import {
  applyThemeChoice,
  type AppliedTheme,
} from "@/features/extension-ui/themes/theme-application";
import {
  PENDING_THEME_CATALOG,
  type ExtensionThemeCatalog,
} from "@/features/extension-ui/themes/theme-catalog";
import { themeIdFromChoice } from "@/features/extension-ui/themes/theme-parser";

export type { ThemeChoice } from "@/lib/app-themes";

const CHOICE_STORAGE_KEY = "clgo-theme";
const BASE_STORAGE_KEY = "clgo-theme-base";

function systemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function safeStoredBase(): ResolvedTheme {
  try {
    const saved = localStorage.getItem(BASE_STORAGE_KEY);
    if (saved === "light" || saved === "dark") return saved;
  } catch {
    // Le système reste l'autorité de repli si le stockage est indisponible.
  }
  return systemPrefersDark() ? "dark" : "light";
}

function isStoredExtensionChoice(value: string | null): value is ExtensionThemeChoice {
  if (!value?.startsWith("extension:")) return false;
  try {
    themeIdFromChoice(value as ExtensionThemeChoice);
    return true;
  } catch {
    return false;
  }
}

function getInitialChoice(): ThemeChoice {
  try {
    const saved = localStorage.getItem(CHOICE_STORAGE_KEY);
    if (isCoreThemeChoice(saved) || isStoredExtensionChoice(saved)) return saved;
  } catch {
    // La valeur corrompue ou inaccessible retombe sur le système.
  }
  return "system";
}

function initialAppliedTheme(choice: ThemeChoice): AppliedTheme {
  return isCoreThemeChoice(choice)
    ? resolveTheme(choice, systemPrefersDark())
    : safeStoredBase();
}

function persistTheme(choice: ThemeChoice, base: "light" | "dark"): void {
  try {
    localStorage.setItem(CHOICE_STORAGE_KEY, choice);
    localStorage.setItem(BASE_STORAGE_KEY, base);
  } catch {
    // Le thème reste appliqué même si le stockage local est indisponible.
  }
}

export function useTheme() {
  const initial = useMemo(() => getInitialChoice(), []);
  const [choice, setChoiceState] = useState<ThemeChoice>(initial);
  const [theme, setThemeState] = useState<AppliedTheme>(() => initialAppliedTheme(initial));
  const [catalog, setCatalog] = useState<ExtensionThemeCatalog>(PENDING_THEME_CATALOG);
  const unavailableChoice = useRef<ExtensionThemeChoice | null>(null);
  const { show } = useToast();
  const { t } = useTranslation();

  useEffect(() => {
    if (!isCoreThemeChoice(choice) && !catalog.ready) return;
    const applied = applyThemeChoice(
      document.documentElement,
      choice,
      catalog,
      systemPrefersDark(),
    );
    if (applied) {
      unavailableChoice.current = null;
      // eslint-disable-next-line react-hooks/set-state-in-effect -- dérivé du catalogue validé
      setThemeState(applied);
      const base = applied.startsWith("extension:")
        ? catalog.byChoice.get(applied as ExtensionThemeChoice)!.colorScheme
        : getThemeColorScheme(applied as ResolvedTheme);
      persistTheme(choice, base);
      return;
    }
    if (unavailableChoice.current !== choice) {
      unavailableChoice.current = choice as ExtensionThemeChoice;
      show(t("extensions.ui.themeUnavailable"), "warning");
    }
    setChoiceState("system");
  }, [catalog, choice, show, t]);

  useEffect(() => {
    if (choice !== "system") return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => {
      const applied = applyThemeChoice(document.documentElement, "system", catalog, media.matches);
      if (!applied) return;
      setThemeState(applied);
      persistTheme("system", getThemeColorScheme(applied as ResolvedTheme));
    };
    media.addEventListener("change", handleChange);
    return () => media.removeEventListener("change", handleChange);
  }, [catalog, choice]);

  const setTheme = useCallback((next: ThemeChoice) => {
    const available = catalog.entries.map((entry) => entry.choice);
    if (isThemeChoice(next, available)) setChoiceState(next);
  }, [catalog.entries]);

  const setThemeCatalog = useCallback((next: ExtensionThemeCatalog) => {
    setCatalog(next);
  }, []);

  const toggle = useCallback(() => {
    const choices: ThemeChoice[] = [
      ...THEME_OPTIONS.slice(0, -1).map(({ id }) => id),
      ...catalog.entries.map((entry) => entry.choice),
      "system",
    ];
    setChoiceState((current) => getNextThemeChoice(current, choices));
  }, [catalog.entries]);

  return { theme, choice, setTheme, setThemeCatalog, toggle } as const;
}
