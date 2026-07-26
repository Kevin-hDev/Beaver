import { useMemo, useState, useRef, useEffect } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { CaretDown, ChatCircleDots, Check, Hand, ShieldWarning } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import { floatingMenuPortalRoot, useFloatingMenuPosition } from "@/hooks/use-floating-menu-position";
import { focusLocalListItem, useLocalListNavigation } from "@/hooks/use-local-list-navigation";
import "./permission-mode-selector.css";

interface Props {
  mode: PermissionMode;
  availableModes?: PermissionMode[];
  onChange: (mode: PermissionMode) => void;
}

const MODES: PermissionMode[] = ["chat", "manual", "auto"];

export function PermissionModeSelector({ mode, availableModes = MODES, onChange }: Props) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const { anchorRef, floatingRef, floatingStyle } = useFloatingMenuPosition(open, "left", 6);
  const navItems = useMemo(() => availableModes.map((m) => ({
    id: modeNavId(m),
    onSelect: () => {
      onChange(m);
      setOpen(false);
    },
  })), [availableModes, onChange]);
  const { activate, getItemRef, isActive, listProps } = useLocalListNavigation({
    items: navItems,
    enabled: open,
    selectedId: modeNavId(mode),
    onEscape: () => setOpen(false),
  });

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      const target = e.target as Node;
      if (rootRef.current?.contains(target) || floatingRef.current?.contains(target)) return;
      setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      const pressed = e.key;
      if (pressed.startsWith("Esc")) {
        setOpen(false);
        return;
      }
      switch (pressed) {
        case "1":
          if (availableModes.includes("chat")) onChange("chat");
          setOpen(false);
          break;
        case "2":
          if (availableModes.includes("manual")) onChange("manual");
          setOpen(false);
          break;
        case "3":
          if (availableModes.includes("auto")) onChange("auto");
          setOpen(false);
          break;
      }
    };
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [availableModes, floatingRef, open, onChange]);

  useEffect(() => {
    if (!open) return;
    requestAnimationFrame(() => focusLocalListItem(floatingRef.current, 1));
  }, [floatingRef, open]);

  const label = t(`permissionMode.${mode}Label`);
  const portalRoot = floatingMenuPortalRoot();

  return (
    <div className="perm-mode-root" ref={rootRef} data-keyboard-scope={open ? "local" : undefined}>
      <Tooltip label={t("permissionMode.toggleHint")}>
        <button
          ref={(node) => { anchorRef.current = node; }}
          type="button"
          className={`perm-mode-trigger perm-mode-trigger-${mode}`}
          onClick={() => setOpen((v) => !v)}
          onKeyDown={(event) => {
            if (!open && (event.key === "Enter" || event.key === " " || event.key === "ArrowDown")) {
              setOpen(true);
              return;
            }
            if (open) listProps.onKeyDown(event);
          }}
        >
          <ModeIcon mode={mode} className="perm-mode-trigger-icon" size="var(--icon-lg)" />
          <span className={`perm-mode-text perm-mode-${mode}`}>{label}</span>
          <CaretDown size="var(--icon-2xs)" className="perm-mode-caret" />
        </button>
      </Tooltip>

      {open && createPortal(
        <div
          ref={floatingRef}
          className="perm-mode-dropdown"
          data-keyboard-scope="local"
          role="menu"
          tabIndex={-1}
          style={floatingStyle}
          onKeyDown={listProps.onKeyDown}
        >
          {availableModes.map((m) => {
            const navId = modeNavId(m);
            return (
              <button
                key={m}
                type="button"
                role="menuitem"
                className={`perm-mode-option perm-mode-option-${m}`}
                ref={getItemRef(navId)}
                tabIndex={isActive(navId) ? 0 : -1}
                data-local-nav-item="true"
                data-local-nav-active={isActive(navId) ? "true" : undefined}
                onFocus={() => activate(navId)}
                onMouseEnter={() => activate(navId)}
                onClick={() => { onChange(m); setOpen(false); }}
              >
                <ModeIcon mode={m} />
                <span className="perm-mode-option-copy">
                  <span className={`perm-mode-option-label perm-mode-${m}`}>
                    {t(`permissionMode.${m}Label`)}
                  </span>
                  <span className="perm-mode-option-description">
                    {t(`permissionMode.${m}Description`)}
                  </span>
                </span>
                <span className="perm-mode-option-right">
                  {m === mode && <Check size="var(--icon-lg)" />}
                </span>
              </button>
            );
          })}
        </div>,
        portalRoot,
      )}
    </div>
  );
}

function ModeIcon({
  mode,
  className = "perm-mode-option-icon",
  size = "var(--icon-2xl)",
}: {
  mode: PermissionMode;
  className?: string;
  size?: number | string;
}) {
  const props = { size, weight: "regular" as const, className };
  if (mode === "chat") return <ChatCircleDots {...props} />;
  if (mode === "manual") return <Hand {...props} />;
  return <ShieldWarning {...props} />;
}

function modeNavId(mode: PermissionMode) {
  return `mode:${mode}`;
}
