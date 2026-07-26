import type { KeyboardEvent, RefObject } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import { validateBranchName } from "@/lib/branch-name";
import { branchCreateErrorKey } from "./branch-selector-utils";

interface BranchSelectorCreateFormProps {
  inputRef: RefObject<HTMLInputElement | null>;
  value: string;
  error: string;
  isCreating: boolean;
  placeholder: string;
  onValueChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

export function BranchSelectorCreateForm({
  inputRef,
  value,
  error,
  isCreating,
  placeholder,
  onValueChange,
  onSubmit,
  onCancel,
}: BranchSelectorCreateFormProps) {
  const { t } = useTranslation();
  const validation = validateBranchName(value.trim());
  const inlineError = error || (!validation.valid && value.trim()
    ? t(branchCreateErrorKey(validation.reason))
    : "");
  const submitDisabled = isCreating || !validation.valid;

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter" && !submitDisabled) onSubmit();
    if (event.key === "Escape") {
      event.stopPropagation();
      if (!isCreating) onCancel();
    }
  };

  return (
    <div className="bs-create-form">
      <div className="bs-create-row">
        <input
          ref={inputRef}
          className="bs-create-input"
          placeholder={placeholder}
          value={value}
          disabled={isCreating}
          onChange={(event) => onValueChange(event.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button
          type="button"
          className="icon-btn bs-create-submit"
          aria-label={t("branches.createSubmit")}
          disabled={submitDisabled}
          onClick={onSubmit}
        >
          <Check size="var(--icon-sm)" weight="bold" />
        </button>
      </div>
      {inlineError && <div className="bs-create-error">{inlineError}</div>}
    </div>
  );
}
