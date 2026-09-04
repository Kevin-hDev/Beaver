import { createContext, useContext, useEffect, useMemo, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useExtensions } from "@/hooks/use-extensions";
import { useStandardCatalog } from "../standard/catalog-context";
import {
  buildThemeCatalog,
  PENDING_THEME_CATALOG,
  type ExtensionThemeCatalog,
} from "./theme-catalog";

const ThemeCatalogContext = createContext<ExtensionThemeCatalog>(PENDING_THEME_CATALOG);

export function ThemeCatalogProvider({
  children,
  onCatalogChange,
}: {
  children: ReactNode;
  onCatalogChange: (catalog: ExtensionThemeCatalog) => void;
}) {
  const standard = useStandardCatalog();
  const { extensions } = useExtensions();
  const { i18n } = useTranslation();
  const ready = standard.state.kind === "empty" || standard.state.kind === "ready"
    || standard.state.kind === "stale-error";
  const names = useMemo(() => new Map(extensions.map((record) => [
    record.manifest.id,
    record.manifest.name,
  ])), [extensions]);
  const catalog = useMemo(() => buildThemeCatalog(
    standard.snapshot,
    names,
    i18n.resolvedLanguage ?? i18n.language,
    ready,
  ), [i18n.language, i18n.resolvedLanguage, names, ready, standard.snapshot]);

  useEffect(() => {
    onCatalogChange(catalog);
  }, [catalog, onCatalogChange]);

  return <ThemeCatalogContext.Provider value={catalog}>{children}</ThemeCatalogContext.Provider>;
}

export function useThemeCatalog(): ExtensionThemeCatalog {
  return useContext(ThemeCatalogContext);
}
