import { useState } from "react";
import { BrowserIcon } from "./browser-icon";

export function BrowserTabFavicon({ pngBase64 }: { pngBase64?: string }) {
  return pngBase64
    ? <FaviconImage key={pngBase64} pngBase64={pngBase64} />
    : <BrowserIcon className="ib-tab-icon" />;
}

function FaviconImage({ pngBase64 }: { pngBase64: string }) {
  const [failed, setFailed] = useState(false);
  if (failed) return <BrowserIcon className="ib-tab-icon" />;
  return <img className="ib-tab-icon" src={`data:image/png;base64,${pngBase64}`}
    alt="" aria-hidden="true" draggable={false} onError={() => setFailed(true)} />;
}
