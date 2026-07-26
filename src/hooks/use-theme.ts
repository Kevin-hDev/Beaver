import { useState, useCallback, useEffect } from "react";
import {
  getNextThemeChoice,
  getThemeColorScheme,
  isThemeChoice,
  resolveTheme,
  type ResolvedTheme,
  type ThemeChoice,
} from "@/lib/app-themes";

export type { ThemeChoice } from "@/lib/app-themes";

function systemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function getInitialChoice(): ThemeChoice {
  try {
    const saved = localStorage.getItem("clgo-theme");
    return isThemeChoice(saved) ? saved : "system";
  } catch {
    return "system";
  }
}

function persistThemeChoice(choice: ThemeChoice): void {
  try {
    localStorage.setItem("clgo-theme", choice);
  } catch {
    // Le thème reste appliqué même si le stockage local est indisponible.
  }
}

function applyTheme(theme: ResolvedTheme): void {
  document.documentElement.setAttribute("data-theme", getThemeColorScheme(theme));
  document.documentElement.setAttribute("data-palette", theme);
}

export function useTheme() {
  const [choice, setChoiceState] = useState<ThemeChoice>(getInitialChoice);
  const [resolved, setResolved] = useState<ResolvedTheme>(() =>
    resolveTheme(getInitialChoice(), systemPrefersDark()),
  );

  useEffect(() => {
    const nextTheme = resolveTheme(choice, systemPrefersDark());
    // eslint-disable-next-line react-hooks/set-state-in-effect -- derived theme resolution on choice change is intentional
    setResolved(nextTheme);
    applyTheme(nextTheme);
    persistThemeChoice(choice);
  }, [choice]);

  useEffect(() => {
    if (choice !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      const nextTheme = resolveTheme("system", mq.matches);
      setResolved(nextTheme);
      applyTheme(nextTheme);
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [choice]);

  const setTheme = useCallback((t: ThemeChoice) => {
    setChoiceState(t);
  }, []);

  const toggle = useCallback(() => {
    setChoiceState(getNextThemeChoice);
  }, []);

  return { theme: resolved, choice, setTheme, toggle } as const;
}
