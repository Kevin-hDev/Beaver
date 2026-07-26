import { useTranslation } from "react-i18next";
import {
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  Maximize2,
  Minimize2,
  RotateCw,
  Square,
} from "@/components/ui/icons";
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
          className="ib-address-input"
          value={props.address}
          maxLength={MAX_BROWSER_URL_LENGTH}
          placeholder={t("browser.addressPlaceholder")}
          aria-invalid={props.invalid}
          onFocus={props.onAddressFocus}
          onBlur={props.onAddressBlur}
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
          : <Maximize2 size="var(--icon-md)" />}
      </button>
    </div>
  );
}
