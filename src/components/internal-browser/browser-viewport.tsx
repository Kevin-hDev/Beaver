import { useTranslation } from "react-i18next";
import { BrowserHome } from "./browser-home";
import type { LocalSite } from "./browser-types";

interface BrowserViewportProps {
  setSurfaceElement: (element: HTMLDivElement | null) => void;
  loading: boolean;
  homeVisible: boolean;
  nativeActive: boolean;
  sites: LocalSite[];
  statusKey: string | null;
  onOpenLocalSite: (url: string) => void;
}

export function BrowserViewport({
  setSurfaceElement,
  loading,
  homeVisible,
  nativeActive,
  sites,
  statusKey,
  onOpenLocalSite,
}: BrowserViewportProps) {
  const { t } = useTranslation();
  return (
    <>
      {statusKey && <p className="ib-status" role="alert">{t(statusKey)}</p>}
      <div className="ib-surface" data-native-active={nativeActive} ref={setSurfaceElement}>
        {loading && <p className="ib-status">{t("browser.loading")}</p>}
        {homeVisible && <BrowserHome sites={sites} onOpen={onOpenLocalSite} />}
      </div>
    </>
  );
}
