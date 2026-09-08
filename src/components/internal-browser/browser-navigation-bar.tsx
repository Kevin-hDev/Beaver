import { useEffect, useLayoutEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  Minimize2,
  RotateCw,
  Square,
} from "@/components/ui/icons";
import { FullscreenIcon } from "@/components/ui/panel-action-icons";
import { MAX_BROWSER_URL_LENGTH, type BrowserTabState } from "./browser-types";

interface BrowserNavigationBarProps {
  tab: BrowserTabState;
  address: string;
  invalid: boolean;
  fullscreen: boolean;
  onAddressFocus: () => void;
  onAddressBlur: () => void;
  onAddressChange: (value: string) => void;
  onSubmit: () => void;
  onAction: (action: "back" | "forward" | "reloadOrStop") => void;
  onFullscreenChange: (fullscreen: boolean) => void;
}

export function BrowserNavigationBar(props: BrowserNavigationBarProps) {
  const { t } = useTranslation();
  /* Le clic qui donne le focus replace parfois le curseur après onFocus. CEF
     peut ensuite réduire une sélection déjà complète : on garde une réparation
     jusqu'au premier selectionchange non complet, sans toucher aux clics suivants. */
  const selectAfterPointer = useRef(false);
  const addressRef = useRef<HTMLInputElement>(null);
  const tabIdRef = useRef(props.tab.id);
  const repairAfterNativeClick = useRef<{ tabId: string; value: string } | null>(null);

  useLayoutEffect(() => {
    tabIdRef.current = props.tab.id;
    repairAfterNativeClick.current = null;
  }, [props.address, props.tab.id]);

  useEffect(() => {
    const repairNativeSelection = () => {
      const repair = repairAfterNativeClick.current;
      const address = addressRef.current;
      if (!repair || !address || repair.tabId !== tabIdRef.current ||
        repair.value !== address.value || document.activeElement !== address || !document.hasFocus()) {
        repairAfterNativeClick.current = null;
        return;
      }
      if (address.selectionStart === 0 && address.selectionEnd === address.value.length) return;
      repairAfterNativeClick.current = null;
      address.setSelectionRange(0, address.value.length);
    };
    const cancelNativeSelectionRepair = () => { repairAfterNativeClick.current = null; };
    document.addEventListener("selectionchange", repairNativeSelection);
    window.addEventListener("blur", cancelNativeSelectionRepair);
    return () => {
      repairAfterNativeClick.current = null;
      document.removeEventListener("selectionchange", repairNativeSelection);
      window.removeEventListener("blur", cancelNativeSelectionRepair);
    };
  }, []);

  const iconButton = (
    label: string,
    disabled: boolean,
    action: "back" | "forward" | "reloadOrStop",
    icon: React.ReactNode,
  ) => (
    <button
      className="ib-nav-button"
      type="button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={() => props.onAction(action)}
    >
      {icon}
    </button>
  );

  return (
    <div className="ib-navigation-bar">
      <div className="ib-navigation-actions">
        {iconButton(t("browser.back"), !props.tab.canGoBack, "back", <ArrowLeft size="var(--icon-md)" />)}
        {iconButton(t("browser.forward"), !props.tab.canGoForward, "forward", <ArrowRight size="var(--icon-md)" />)}
        {iconButton(
          props.tab.loading ? t("browser.stop") : t("browser.reload"),
          props.tab.url === null,
          "reloadOrStop",
          props.tab.loading ? <Square size="var(--icon-sm)" /> : <RotateCw size="var(--icon-md)" />,
        )}
      </div>
      <form className="ib-address-form" aria-label={t("browser.addressLabel")} onSubmit={(event) => {
        event.preventDefault();
        props.onSubmit();
      }}>
        <input
          ref={addressRef}
          className="ib-address-input"
          value={props.address}
          maxLength={MAX_BROWSER_URL_LENGTH}
          placeholder={t("browser.addressPlaceholder")}
          aria-invalid={props.invalid}
          onPointerDown={(event) => {
            repairAfterNativeClick.current = null;
            /* Une vue CEF native peut prendre le focus macOS sans remettre à
               zéro activeElement dans le document Tauri. hasFocus est alors
               la seule autorité qui indique que ce clic entre dans le champ. */
            selectAfterPointer.current = !document.hasFocus() ||
              document.activeElement !== event.currentTarget;
          }}
          onPointerCancel={() => {
            selectAfterPointer.current = false;
            repairAfterNativeClick.current = null;
          }}
          onFocus={(event) => {
            props.onAddressFocus();
            event.currentTarget.select();
          }}
          onBlur={() => {
            selectAfterPointer.current = false;
            repairAfterNativeClick.current = null;
            props.onAddressBlur();
          }}
          onClick={(event) => {
            const shouldSelect = selectAfterPointer.current;
            selectAfterPointer.current = false;
            if (shouldSelect) {
              event.currentTarget.select();
              repairAfterNativeClick.current = {
                tabId: props.tab.id,
                value: event.currentTarget.value,
              };
            }
          }}
          onKeyDown={() => { repairAfterNativeClick.current = null; }}
          onInput={() => { repairAfterNativeClick.current = null; }}
          onChange={(event) => props.onAddressChange(event.target.value)}
        />
        <button
          className="ib-address-go"
          type="submit"
          aria-label={t("browser.openAddress")}
          onMouseDown={(event) => event.preventDefault()}
        >
          <ArrowUpRight size="var(--icon-md)" aria-hidden="true" />
        </button>
      </form>
      <button
        className="ib-nav-button"
        type="button"
        aria-label={props.fullscreen ? t("filePreview.reduce") : t("filePreview.fullscreen")}
        title={props.fullscreen ? t("filePreview.reduce") : t("filePreview.fullscreen")}
        onClick={() => props.onFullscreenChange(!props.fullscreen)}
      >
        {props.fullscreen
          ? <Minimize2 size="var(--icon-md)" />
          : <FullscreenIcon />}
      </button>
    </div>
  );
}
