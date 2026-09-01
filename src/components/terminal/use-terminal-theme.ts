import { useEffect, useState } from "react";
import type { ITheme } from "@xterm/xterm";
import { readTerminalTheme } from "./terminal-theme";

export function useTerminalTheme(): ITheme {
  const [theme, setTheme] = useState<ITheme>(() => readTerminalTheme());

  useEffect(() => {
    const observer = new MutationObserver(() => setTheme(readTerminalTheme()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => observer.disconnect();
  }, []);

  return theme;
}
