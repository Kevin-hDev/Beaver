import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Brain, CaretDown, Check } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useKeyboard } from "@/hooks/use-keyboard";
import {
  floatingMenuPortalRoot,
  useFloatingMenuPosition,
} from "@/hooks/use-floating-menu-position";
import type { AvailableModel } from "@/hooks/use-available-models";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import { cn } from "@/lib/utils";
import "./reasoning-selector.css";

interface ReasoningSelectorProps {
  model: AvailableModel | null;
  reasoningMode?: string | null;
  onChange: (mode: ReasoningMode) => void;
  align?: "left" | "right";
}

export function ReasoningSelector({
  model,
  reasoningMode,
  onChange,
  align = "left",
}: ReasoningSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const { anchorRef, floatingRef, floatingStyle } = useFloatingMenuPosition(open, align, 4);
  const options = reasoningModeOptions(model);
  const selectedMode = normalizeReasoningMode(
    reasoningMode,
    options,
    model?.default_reasoning_mode,
  );
  const selectedOption = options.find((option) => option.mode === selectedMode) ?? options[0];

  useKeyboard({ onEscape: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const onDocumentMouseDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (rootRef.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", onDocumentMouseDown);
    return () => document.removeEventListener("mousedown", onDocumentMouseDown);
  }, [floatingRef, open]);

  if (!selectedOption) return null;

  const selectedLabel = t(selectedOption.labelKey);
  const dropdown = open ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      className="rs-dropdown"
      aria-label={t("agentLocal.reasoningTitle")}
    >
      {options.map((option) => (
        <button
          key={option.mode}
          type="button"
          className={cn(
            "rs-option",
            selectedMode === option.mode && "rs-option-active",
          )}
          onClick={() => {
            onChange(option.mode);
            setOpen(false);
          }}
        >
          <span>{t(option.labelKey)}</span>
          {selectedMode === option.mode && <Check size="var(--icon-xs)" />}
        </button>
      ))}
    </div>
  ) : null;

  return (
    <div ref={rootRef} className="rs-root">
      <Tooltip label={t("agentLocal.reasoningTitle")} align="right">
        <button
          ref={(node) => { anchorRef.current = node; }}
          type="button"
          className="rs-trigger"
          aria-expanded={open}
          aria-label={`${t("agentLocal.reasoningTitle")}: ${selectedLabel}`}
          onClick={() => setOpen((current) => !current)}
        >
          <Brain size="var(--icon-xs)" className="rs-trigger-icon" />
          <span className="rs-trigger-label">{selectedLabel}</span>
          <CaretDown size="var(--icon-2xs)" className="rs-trigger-caret" />
        </button>
      </Tooltip>
      {dropdown ? createPortal(dropdown, floatingMenuPortalRoot()) : null}
    </div>
  );
}
