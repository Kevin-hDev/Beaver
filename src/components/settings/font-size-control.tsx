import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import {
  clampFontSizePx,
  FONT_SIZE_MAX,
  FONT_SIZE_MIN,
  type FontSize,
} from "@/hooks/use-settings";
import "./font-size-control.css";

interface FontSizeControlProps {
  value: FontSize;
  onChange: (value: FontSize) => void;
}

export function FontSizeControl({ value, onChange }: FontSizeControlProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(String(value));

  const commit = (raw: string) => {
    const next = clampFontSizePx(Number(raw));
    setDraft(String(next));
    onChange(next);
  };

  const updateDraft = (raw: string) => {
    setDraft(raw);
    const parsed = Number(raw);
    if (Number.isFinite(parsed) && parsed >= FONT_SIZE_MIN && parsed <= FONT_SIZE_MAX) {
      onChange(parsed);
    }
  };

  const step = (delta: number) => {
    const next = clampFontSizePx(value + delta);
    setDraft(String(next));
    onChange(next);
  };

  return (
    <div className="fsc-control">
      <input
        className="fsc-input"
        type="number"
        min={FONT_SIZE_MIN}
        max={FONT_SIZE_MAX}
        step={1}
        value={draft}
        onChange={(event) => updateDraft(event.currentTarget.value)}
        onBlur={() => commit(draft)}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "ArrowUp") {
            event.preventDefault();
            step(1);
          }
          if (event.key === "ArrowDown") {
            event.preventDefault();
            step(-1);
          }
        }}
      />
      <div className="fsc-stepper" aria-hidden="false">
        <button
          type="button"
          className="fsc-step-btn"
          aria-label={t("settings.general.fontSizeIncrease")}
          onClick={() => step(1)}
        >
          <CaretUp size="var(--icon-xs)" weight="fill" />
        </button>
        <button
          type="button"
          className="fsc-step-btn"
          aria-label={t("settings.general.fontSizeDecrease")}
          onClick={() => step(-1)}
        >
          <CaretDown size="var(--icon-xs)" weight="fill" />
        </button>
      </div>
      <span className="fsc-unit">px</span>
    </div>
  );
}
