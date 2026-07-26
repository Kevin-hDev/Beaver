import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "react-i18next";
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

export function LinkPreviewCard({ url }: { url: string }) {
  const { t } = useTranslation();
  const [data, setData] = useState<LinkPreviewData | null | undefined>(
    cache.has(url) ? cache.get(url) : undefined,
  );
  const [imgError, setImgError] = useState(false);

  useEffect(() => {
    if (cache.has(url)) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- cache hit, synchronous setState is intentional
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

  if (data === undefined) {
    return (
      <div className="lpc-card lpc-loading">
        <div className="lpc-skeleton-img" />
        <div className="lpc-body">
          <div className="lpc-skeleton-line lpc-skeleton-short" />
          <div className="lpc-skeleton-line" />
        </div>
      </div>
    );
  }

  if (!data) return null;

  const title = data.title || t("linkPreview.noTitle");
  const siteName = data.site_name || data.domain;
  const showImage = data.image && !imgError;

  return (
    <button
      className="lpc-card"
      onClick={() => void open(url)}
      title={t("linkPreview.openSite")}
      type="button"
    >
      {showImage && (
        <img
          className="lpc-image"
          src={data.image!}
          alt=""
          onError={() => setImgError(true)}
        />
      )}
      <div className="lpc-body">
        <div className="lpc-header">
          {data.favicon && (
            <img
              className="lpc-favicon"
              src={data.favicon}
              alt=""
              onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
            />
          )}
          <span className="lpc-site-name">{siteName}</span>
        </div>
        <div className="lpc-title">{title}</div>
        {data.description && (
          <div className="lpc-description">{data.description}</div>
        )}
        <div className="lpc-url">{url}</div>
      </div>
    </button>
  );
}
