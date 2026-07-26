import { useTranslation } from "react-i18next";
import type { ForecastDocPage } from "./forecast-docs-types";

interface ForecastDocsSidebarProps {
  pages: ForecastDocPage[];
  activeId: string;
  onSelect: (id: string) => void;
}

export function ForecastDocsSidebar({
  pages,
  activeId,
  onSelect,
}: ForecastDocsSidebarProps) {
  const { t } = useTranslation();

  return (
    <aside className="fd-sidebar">
      <div className="fd-sidebar-kicker">Forecast</div>
      <nav className="fd-nav" aria-label={t("forecast.docs.navLabel")}>
        {pages.map((page) => (
          <button
            key={page.id}
            className={`fd-nav-item ${page.id === activeId ? "fd-nav-item-active" : ""}`}
            type="button"
            onClick={() => onSelect(page.id)}
          >
            {t(page.navLabel)}
          </button>
        ))}
      </nav>
    </aside>
  );
}
