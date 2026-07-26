import { useState } from "react";
import { useTranslation } from "react-i18next";
import "./api-key-secret-input.css";

interface ApiKeySecretInputProps {
  id?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  className?: string;
  inputClassName?: string;
  autoFocus?: boolean;
  disabled?: boolean;
  required?: boolean;
}

export function ApiKeySecretInput({
  id,
  value,
  onChange,
  placeholder,
  className,
  inputClassName,
  autoFocus,
  disabled,
  required,
}: ApiKeySecretInputProps) {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  const label = visible ? t("apiKeys.dialog.hideKey") : t("apiKeys.dialog.showKey");

  return (
    <div className={["aksi-field", className ?? ""].filter(Boolean).join(" ")}>
      <input
        id={id}
        type={visible ? "text" : "password"}
        className={inputClassName}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        autoFocus={autoFocus}
        disabled={disabled}
        required={required}
      />
      <button
        type="button"
        className="aksi-toggle"
        aria-label={label}
        onClick={() => setVisible((current) => !current)}
        disabled={disabled}
      >
        {label}
      </button>
    </div>
  );
}
