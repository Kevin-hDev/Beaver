import { useEffect, useState } from "react";
import type { ITheme } from "@xterm/xterm";
import { readTerminalTheme } from "./terminal-theme";

/* Changer de thème pose les deux attributs. `data-theme` ne dit que le clair ou
   le sombre : passer d'une palette sombre à une autre ne le fait pas bouger, et
   le terminal gardait alors les couleurs de la palette précédente. */
const THEME_ATTRIBUTES = ["data-theme", "data-palette"];

export function useTerminalTheme(): ITheme {
  const [theme, setTheme] = useState<ITheme>(() => readTerminalTheme());

  useEffect(() => {
    const observer = new MutationObserver(() => setTheme(readTerminalTheme()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: THEME_ATTRIBUTES,
    });
    return () => observer.disconnect();
  }, []);

  return theme;
}
