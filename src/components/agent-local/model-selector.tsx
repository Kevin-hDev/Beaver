import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { useKeyboard } from "@/hooks/use-keyboard";
import { Tooltip } from "@/components/ui/tooltip";
import { floatingMenuPortalRoot, useFloatingMenuPosition } from "@/hooks/use-floating-menu-position";
import { focusLocalListItem } from "@/hooks/use-local-list-navigation";
import { CaretDown, MagnifyingGlass } from "@/components/ui/icons";
import type { AvailableModel } from "@/hooks/use-available-models";
import { useFavoriteModels } from "@/hooks/use-favorite-models";
import { ModelSelectorList } from "./model-selector-list";
import "./model-selector.css";
import "./model-selector-controls.css";

interface ModelSelectorProps {
  groups: Map<string, AvailableModel[]>;
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
  align?: "left" | "right";
}

export function ModelSelector({
  groups,
  selectedModel,
  selectedProvider,
  onSelect,
  align = "left",
}: ModelSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const { anchorRef, floatingRef, floatingStyle, updateFloatingPosition } =
    useFloatingMenuPosition(open, align, 4);
  const { favorites, isFavorite, toggle: toggleFav } = useFavoriteModels();
  const selectedEntry = groups
    .get(selectedProvider)
    ?.find((model) => model.id === selectedModel);
  const selectedLabel = selectedEntry?.display_name ?? selectedModel;

  useKeyboard({ onEscape: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const onDoc = (event: MouseEvent) => {
      const target = event.target as Node;
      if (ref.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [floatingRef, open]);

  const filteredGroups = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return groups;
    const out = new Map<string, AvailableModel[]>();
    for (const [key, models] of groups.entries()) {
      const kept = models.filter((model) =>
        model.id.toLowerCase().includes(q)
        || model.display_name?.toLowerCase().includes(q),
      );
      if (kept.length > 0) out.set(key, kept);
    }
    return out;
  }, [groups, query]);
  const focusDropdownList = (direction: 1 | -1) => {
    focusLocalListItem(floatingRef.current, direction);
  };

  const dropdown = open ? (
    <div
      ref={floatingRef}
      style={floatingStyle}
      data-keyboard-scope="local"
      className="ms-dropdown"
    >
      <div className="ms-main">
        <div className="ms-search">
          <MagnifyingGlass size="var(--icon-sm)" className="ms-search-icon" />
          <input
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              requestAnimationFrame(updateFloatingPosition);
            }}
            onKeyDown={(event) => {
              if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
              event.preventDefault();
              focusDropdownList(event.key === "ArrowDown" ? 1 : -1);
            }}
            placeholder={t("agentLocal.modelSearch")}
            className="ms-search-input"
            autoFocus
          />
        </div>

        <div className="ms-list">
          <ModelSelectorList
            groups={filteredGroups}
            favorites={favorites}
            isFavorite={isFavorite}
            onToggleFavorite={(p, m) => void toggleFav(p, m)}
            selectedModel={selectedModel}
            selectedProvider={selectedProvider}
            onSelect={(model, provider) => {
              onSelect(model, provider);
              setOpen(false);
              setQuery("");
            }}
          />
        </div>
      </div>
    </div>
  ) : null;
  const portalRoot = floatingMenuPortalRoot();

  return (
    <div
      ref={ref}
      className={`ms-root${align === "right" ? " ms-root-align-right" : ""}`}
      data-keyboard-scope={open ? "local" : undefined}
    >
      <Tooltip label={t("agentLocal.selectModelHint")} align="right">
        <button
          ref={(node) => { anchorRef.current = node; }}
          type="button"
          onClick={() => setOpen(!open)}
          onKeyDown={(event) => {
            if (!open && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
              setOpen(true);
              requestAnimationFrame(() => focusDropdownList(event.key === "ArrowDown" ? 1 : -1));
              return;
            }
            if (open && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
              event.preventDefault();
              focusDropdownList(event.key === "ArrowDown" ? 1 : -1);
            }
          }}
          className={`ms-trigger${selectedModel ? "" : " ms-trigger-empty"}`}
        >
          <span className="ms-trigger-label">
            {selectedLabel || t("agentLocal.selectModel")}
          </span>
          <CaretDown size="var(--icon-2xs)" className="ms-trigger-caret" />
        </button>
      </Tooltip>
      {dropdown ? createPortal(dropdown, portalRoot) : null}
    </div>
  );
}
