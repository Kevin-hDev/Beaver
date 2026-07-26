import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { loadForecastDocPages } from "./forecast-docs-data";
import { ForecastDocsAccordion } from "./forecast-docs-accordion";
import { ForecastDocsMarkdown } from "./forecast-docs-markdown";
import { ForecastDocsSidebar } from "./forecast-docs-sidebar";
import type { ForecastDocPage } from "./forecast-docs-types";
import "./forecast-docs.css";

export function ForecastDocsWindow() {
  const { t, i18n } = useTranslation();
  const [pages, setPages] = useState<ForecastDocPage[]>([]);
  const [activeId, setActiveId] = useState("");

  useEffect(() => {
    let cancelled = false;
    void loadForecastDocPages().then((loaded) => {
      if (cancelled) return;
      setPages(loaded);
      setActiveId((prev) => prev && loaded.some((p) => p.id === prev) ? prev : (loaded[0]?.id ?? ""));
    });
    return () => {
      cancelled = true;
    };
  }, [i18n.language]);

  const activePage = useMemo(
    () => pages.find((page) => page.id === activeId) ?? pages[0],
    [pages, activeId],
  );

  if (!activePage) {
    return <main className="fd-window" />;
  }

  return (
    <main className="fd-window">
      <ForecastDocsSidebar pages={pages} activeId={activePage.id} onSelect={setActiveId} />
      <article className="fd-content">
        <header className="fd-header">
          <div>
            <div className="fd-kicker">{t("forecast.docs.kicker")}</div>
            <h1>{activePage.title}</h1>
          </div>
          <span className="fd-status">{t("forecast.docs.status")}</span>
        </header>

        {activePage.summary && (
          <section className="fd-summary">
            <ForecastDocsMarkdown content={activePage.summary} />
          </section>
        )}

        <div className="fd-sections">
          {activePage.sections.map((section, index) => (
            <ForecastDocsAccordion
              key={section.id}
              section={section}
              defaultOpen={index === 0}
            />
          ))}
        </div>
      </article>
    </main>
  );
}
