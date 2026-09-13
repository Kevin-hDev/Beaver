import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import "./link-preview-card.css";

interface LinkPreviewData {
  url: string;
  domain: string;
  site_name: string | null;
  title: string | null;
  description: string | null;
  image: string | null;
  favicon: string | null;
}

const cache = new Map<string, LinkPreviewData | null>();
const MAX_CACHE = 200;

function evictIfFull() {
  if (cache.size >= MAX_CACHE) {
    const first = cache.keys().next().value;
    if (first) cache.delete(first);
  }
}

/* Le domaine est fiable et suffit ; l'adresse complète reste dans l'infobulle. */
export function LinkPreviewCard({ url }: { url: string }) {
  const { t } = useTranslation();
  const [data, setData] = useState<LinkPreviewData | null | undefined>(
    cache.has(url) ? cache.get(url) : undefined,
  );
  const [imgError, setImgError] = useState(false);
  const [faviconError, setFaviconError] = useState(false);

  useEffect(() => {
    if (cache.has(url)) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- lecture synchrone du cache existant
      setData(cache.get(url));
      return;
    }
    let cancelled = false;
    invoke<LinkPreviewData>("fetch_link_preview", { url })
      .then((result) => {
        evictIfFull();
        cache.set(url, result);
        if (!cancelled) setData(result);
      })
      .catch(() => {
        evictIfFull();
        cache.set(url, null);
        if (!cancelled) setData(null);
      });
    return () => { cancelled = true; };
  }, [url]);

  if (data === undefined) return <LinkPreviewLoading />;
  if (!data) return null;

  const title = data.title || t("linkPreview.noTitle");
  const favicon = data.favicon && !faviconError ? data.favicon : null;
  const chip = (className: string) => favicon && (
    <span className={className}>
      <img src={favicon} alt="" onError={() => setFaviconError(true)} />
    </span>
  );

  return (
    <Tooltip label={url}>
      <button className="lpc-card relief elev-rest" onClick={() => void open(url)} type="button">
        {data.image && !imgError ? (
          <span className="lpc-media">
            <img src={data.image} alt="" onError={() => setImgError(true)} />
          </span>
        ) : (
          <span className="lpc-media lpc-no-image">{chip("lpc-chip lpc-chip-large")}</span>
        )}
        <span className="lpc-body">
          <span className="lpc-title">{title}</span>
          <span className="lpc-source">
            {chip("lpc-chip")}
            <span className="lpc-domain">{data.domain}</span>
          </span>
        </span>
      </button>
    </Tooltip>
  );
}

function LinkPreviewLoading() {
  return (
    <div className="lpc-card lpc-loading relief elev-rest" aria-busy="true">
      <span className="lpc-media lpc-wait" />
      <span className="lpc-body">
        <span className="lpc-bar" />
        <span className="lpc-bar" />
        <span className="lpc-bar lpc-bar-short" />
      </span>
    </div>
  );
}
