import { useTranslation } from "react-i18next";
import {
  EXTENSION_INSTALL_LIMITS,
  type ExtensionInstallSource,
} from "@/lib/extension-install";

interface ExtensionSourceFormProps {
  source: ExtensionInstallSource;
  locator: string;
  busy: boolean;
  onLocatorChange: (value: string) => void;
  onSubmit: () => void;
}

export function ExtensionSourceForm(props: ExtensionSourceFormProps) {
  const { t } = useTranslation();
  return (
    <form
      className="exta-source-form"
      onSubmit={(event) => {
        event.preventDefault();
        props.onSubmit();
      }}
    >
      <label htmlFor="exta-source-input">
        {t(`extensions.add.${props.source}Label`)}
      </label>
      <div className="exta-source-row">
        <input
          id="exta-source-input"
          className="form-input"
          value={props.locator}
          disabled={props.busy}
          maxLength={EXTENSION_INSTALL_LIMITS[props.source]}
          placeholder={t(`extensions.add.${props.source}Placeholder`)}
          autoCapitalize="none"
          autoCorrect="off"
          spellCheck={false}
          onChange={(event) => props.onLocatorChange(event.target.value)}
        />
        <button type="submit" className="wk-btn-primary" disabled={props.busy}>
          {t(props.busy ? "extensions.add.installing" : "extensions.add.install")}
        </button>
      </div>
    </form>
  );
}
