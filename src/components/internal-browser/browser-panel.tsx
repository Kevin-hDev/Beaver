import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { BrowserNavigationBar } from "./browser-navigation-bar";
import { BrowserReplaceDialog } from "./browser-replace-dialog";
import { BrowserTabStrip } from "./browser-tab-strip";
import { BrowserViewport } from "./browser-viewport";
import { resolveBrowserAddress } from "./browser-address";
import { useBrowserSession } from "./use-browser-session";
import { useBrowserSurface } from "./use-browser-surface";
import { useBrowserTabCreation } from "./use-browser-tab-creation";
import { useLocalSites } from "./use-local-sites";
import "./browser-shell.css";
import "./browser-home.css";
import "./browser-dialog.css";

const EMPTY_TAB_ID = "00000000000000000000000000000000";

interface BrowserPanelProps {
  conversationId: string;
  active: boolean;
  fullscreen: boolean;
  onFullscreenChange: (fullscreen: boolean) => void;
}

interface AddressDraft {
  tabId: string;
  value: string;
}

export function BrowserPanel(props: BrowserPanelProps) {
  const { t } = useTranslation();
  const {
    session,
    loading,
    error,
    notice,
    popup,
    clearPopup,
    clearNotice,
    createTab,
    activateTab,
    closeTab: closeSessionTab,
    navigate,
    navigationAction,
  } = useBrowserSession(props.conversationId, props.active);
  const [draft, setDraft] = useState<AddressDraft | null>(null);
  const [invalidTabId, setInvalidTabId] = useState<string | null>(null);
  const [surfaceError, setSurfaceError] = useState(false);
  const activeTab = session?.tabs.find((tab) => tab.id === session.activeTabId) ?? null;
  const tabId = activeTab?.id ?? EMPTY_TAB_ID;
  const address = draft?.tabId === tabId ? draft.value : activeTab?.url ?? "";
  const homeVisible = props.active && Boolean(activeTab) && activeTab?.url === null;
  const localSites = useLocalSites(homeVisible);
  const onSurfaceError = useCallback(() => setSurfaceError(true), []);
  const { hostRef, retrySurface } = useBrowserSurface({
    active: props.active && activeTab?.url !== null,
    conversationId: props.conversationId,
    tabId,
    url: activeTab?.url ?? null,
    onError: onSurfaceError,
  });
  const setSurfaceElement = useCallback((element: HTMLDivElement | null) => {
    hostRef.current = element;
  }, [hostRef]);
  const tabCreation = useBrowserTabCreation({
    popup,
    createTab,
    clearPopup,
    clearNotice,
  });

  const selectTab = (nextTabId: string) => {
    clearNotice();
    setDraft(null);
    setInvalidTabId(null);
    void activateTab(nextTabId);
  };

  const closeTab = (closedTabId: string) => {
    clearNotice();
    setDraft(null);
    setInvalidTabId(null);
    void closeSessionTab(closedTabId);
  };

  const submitAddress = () => {
    if (!activeTab) return;
    const url = resolveBrowserAddress(address);
    if (!url) {
      setInvalidTabId(activeTab.id);
      return;
    }
    setInvalidTabId(null);
    setDraft(null);
    clearNotice();
    void navigate(activeTab.id, url);
  };

  const visibleError = error || surfaceError || tabCreation.dialogError || localSites.error;
  const statusKey = notice === "blockedFeature"
    ? "browser.blockedFeature"
    : visibleError || notice === "engineStopped" ? "browser.operationFailed" : null;

  return (
    <section className="ib-root" aria-label={t("browser.title")}>
      {session && (
        <BrowserTabStrip
          tabs={session.tabs}
          activeTabId={session.activeTabId}
          onSelect={selectTab}
          onClose={closeTab}
          onAdd={() => { void tabCreation.requestNewTab(); }}
        />
      )}
      {activeTab && (
        <BrowserNavigationBar
          tab={activeTab}
          address={address}
          invalid={invalidTabId === activeTab.id}
          fullscreen={props.fullscreen}
          onAddressFocus={() => setDraft({ tabId: activeTab.id, value: address })}
          onAddressBlur={() => setDraft(null)}
          onAddressChange={(value) => {
            setInvalidTabId(null);
            setDraft({ tabId: activeTab.id, value });
          }}
          onSubmit={submitAddress}
          onAction={(action) => {
            if (
              action === "reloadOrStop" &&
              (notice === "engineStopped" || surfaceError)
            ) {
              clearNotice();
              setSurfaceError(false);
              retrySurface();
              return;
            }
            clearNotice();
            void navigationAction(activeTab.id, action);
          }}
          onFullscreenChange={props.onFullscreenChange}
        />
      )}
      <BrowserViewport
        setSurfaceElement={setSurfaceElement}
        loading={loading}
        homeVisible={homeVisible}
        nativeActive={Boolean(props.active && activeTab && activeTab.url !== null)}
        sites={localSites.sites}
        statusKey={statusKey}
        onOpenLocalSite={(url) => {
          if (activeTab) {
            clearNotice();
            void navigate(activeTab.id, url);
          }
        }}
      />
      {tabCreation.replacement && (
        <BrowserReplaceDialog
          candidateTitle={tabCreation.replacement.candidateTitle}
          busy={tabCreation.replacing}
          onCancel={tabCreation.closeDialog}
          onConfirm={() => { void tabCreation.confirmReplacement(); }}
        />
      )}
    </section>
  );
}
