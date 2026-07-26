import { useTranslation } from "react-i18next";
import { BrowserIcon } from "./browser-icon";
import type { LocalSite } from "./browser-types";

interface BrowserHomeProps {
  sites: LocalSite[];
  onOpen: (url: string) => void;
}

export function BrowserHome({ sites, onOpen }: BrowserHomeProps) {
  const { t } = useTranslation();
  return (
    <div className="ib-home">
      <BrowserIcon className="ib-home-icon" />
      <h2 className="ib-home-title">{t("browser.startTitle")}</h2>
      <p className="ib-home-description">{t("browser.startDescription")}</p>
      {sites.length > 0 && (
        <div className="ib-local-sites" aria-label={t("browser.localSites")}>
          {sites.map((site) => (
            <button
              className="ib-local-site"
              type="button"
              key={site.port}
              title={site.url}
              onClick={() => onOpen(site.url)}
            >
              <span className="ib-local-site-title">{site.title}</span>
              <span className="ib-local-site-port">:{site.port}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
