import { useCallback, useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus, Image, Plugs, PuzzlePiece, CaretRight, ClipboardText, ArrowsClockwise,
} from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { useConnectors } from "@/hooks/use-connectors";
import { useExtensions } from "@/hooks/use-extensions";
import { ChatPlusConnectorRow } from "./chat-plus-connector-row";
import { ChatPlusPluginMenu, chatPluginShortcuts } from "./chat-plus-plugin-menu";
import { useChatPlusSubmenuPosition } from "./use-chat-plus-submenu-position";
import { ChatPlusCompressionMenu } from "./chat-plus-compression-menu";
import type { CompressionProfileView } from "@/types/compression-profile.generated";
import "./chat-plus-menu.css";

interface ChatPlusMenuProps {
  onFileImport: () => void;
  agentic: boolean;
  planModeEnabled: boolean;
  onPlanModeChange: (enabled: boolean) => void;
  showCompression?: boolean;
  compressionProfiles?: CompressionProfileView[];
  compressionProfilesStatus?: "loading" | "ready" | "error";
  selectedCompressionId?: string;
  onCompressionSelect?: (profileId: string) => Promise<boolean>;
}

const NO_COMPRESSION_PROFILES: CompressionProfileView[] = [];
const NO_COMPRESSION_SELECT = () => Promise.resolve(false);

export function ChatPlusMenu({
  onFileImport,
  agentic,
  planModeEnabled,
  onPlanModeChange,
  showCompression = false,
  compressionProfiles = NO_COMPRESSION_PROFILES,
  compressionProfilesStatus = "ready",
  selectedCompressionId,
  onCompressionSelect = NO_COMPRESSION_SELECT,
}: ChatPlusMenuProps) {
  const { t } = useTranslation();
  const planModeSwitchId = useId();
  const [open, setOpen] = useState(false);
  const [submenu, setSubmenu] = useState<"compression" | "connectors" | "plugins" | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const submenuRef = useRef<HTMLDivElement>(null);
  const { configured, toggleChatEnabled } = useConnectors();
  const extensionRegistry = useExtensions();

  const close = useCallback(() => { setOpen(false); setSubmenu(null); }, []);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") close(); };
    const onClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) close();
    };
    window.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onClick);
    return () => { window.removeEventListener("keydown", onKey); document.removeEventListener("mousedown", onClick); };
  }, [open, close]);

  const handleFileImport = () => { close(); onFileImport(); };

  const connectedItems = configured.filter((c) => c.status === "connected");

  const submenuLeft = useChatPlusSubmenuPosition(
    open,
    submenu,
    menuRef,
    dropdownRef,
    submenuRef,
  );

  return (
    <div className="cpm-wrapper" ref={menuRef}>
      <Tooltip label={t("chatMenu.plusButtonHint")}>
        <button
          className="icon-btn chat-plus-btn"
          aria-label={t("chatMenu.plusButtonHint")}
          onClick={() => setOpen(!open)}
          type="button"
        >
          <Plus size="var(--icon-md)" />
        </button>
      </Tooltip>

      {open && (
        <div className="cpm-dropdown" ref={dropdownRef}>
          <button type="button" className="menu-row cpm-item" onClick={handleFileImport}>
            <Image size="var(--icon-md)" weight="regular" />
            <span>{t("chatMenu.addFile")}</span>
          </button>

          {showCompression && (
            <button
              type="button"
              className={`menu-row cpm-item cpm-has-sub ${submenu === "compression" ? "active" : ""}`}
              onMouseEnter={() => setSubmenu("compression")}
              onFocus={() => setSubmenu("compression")}
              onClick={() => setSubmenu("compression")}
              aria-haspopup="menu"
              aria-expanded={submenu === "compression"}
            >
              <ArrowsClockwise size="var(--icon-md)" weight="regular" />
              <span>{t("chatMenu.compression")}</span>
              <CaretRight size="var(--icon-xs)" className="cpm-caret" />
            </button>
          )}

          {agentic && (
            <>
              <div className="menu-row cpm-item">
                <ClipboardText size="var(--icon-md)" weight="regular" />
                <label className="cpm-switch-copy" htmlFor={planModeSwitchId}>
                  <span>{t("chatMenu.planMode")}</span>
                  <span className="cpm-item-desc">{t("chatMenu.planModeDesc")}</span>
                </label>
                <ToggleSwitch
                  id={planModeSwitchId}
                  checked={planModeEnabled}
                  ariaLabel={t("chatMenu.planMode")}
                  onCheckedChange={onPlanModeChange}
                />
              </div>

              <div className="cpm-separator" />

              <button
                type="button"
                className={`menu-row cpm-item cpm-has-sub ${submenu === "connectors" ? "active" : ""}`}
                onMouseEnter={() => setSubmenu("connectors")}
                onFocus={() => setSubmenu("connectors")}
                onClick={() => setSubmenu("connectors")}
                aria-haspopup="menu"
                aria-expanded={submenu === "connectors"}
              >
                <Plugs size="var(--icon-md)" weight="regular" />
                <span>{t("chatMenu.connectors")}</span>
                <CaretRight size="var(--icon-xs)" className="cpm-caret" />
              </button>

              <button
                type="button"
                className={`menu-row cpm-item cpm-has-sub ${submenu === "plugins" ? "active" : ""}`}
                onMouseEnter={() => setSubmenu("plugins")}
                onFocus={() => setSubmenu("plugins")}
                onClick={() => setSubmenu("plugins")}
                aria-haspopup="menu"
                aria-expanded={submenu === "plugins"}
              >
                <PuzzlePiece size="var(--icon-md)" weight="regular" />
                <span>{t("chatMenu.plugins")}</span>
                <CaretRight size="var(--icon-xs)" className="cpm-caret" />
              </button>
            </>
          )}
        </div>
      )}

      {open && agentic && submenu === "connectors" && (
        <div
          ref={submenuRef}
          className="cpm-submenu"
          role="group"
          aria-label={t("chatMenu.connectors")}
          style={{ left: submenuLeft }}
          onMouseLeave={() => setSubmenu(null)}
        >
          {connectedItems.length === 0 ? (
            <div className="cpm-sub-empty">{t("chatMenu.noConnectors")}</div>
          ) : (
            connectedItems.map((c) => (
              <ChatPlusConnectorRow
                key={c.id}
                connectorId={c.id}
                displayName={c.display_name}
                enabled={c.enabled_in_chat}
                onToggle={() => void toggleChatEnabled(c.id)}
              />
            ))
          )}
        </div>
      )}

      {open && submenu === "compression" && (
        <div
          ref={submenuRef}
          className="cpm-submenu"
          role="group"
          aria-label={t("chatMenu.compression")}
          style={{ left: submenuLeft }}
          onMouseLeave={() => setSubmenu(null)}
        >
          <ChatPlusCompressionMenu
            profiles={compressionProfiles}
            status={compressionProfilesStatus}
            selectedId={selectedCompressionId}
            onSelect={onCompressionSelect}
            onConfirmed={close}
          />
        </div>
      )}

      {open && agentic && submenu === "plugins" && (
        <div
          ref={submenuRef}
          className="cpm-submenu"
          role="group"
          aria-label={t("chatMenu.plugins")}
          style={{ left: submenuLeft }}
          onMouseLeave={() => setSubmenu(null)}
        >
          <ChatPlusPluginMenu
            extensions={chatPluginShortcuts(extensionRegistry.extensions)}
            busyIds={extensionRegistry.busyIds}
            onToggle={(id, enabled) => void extensionRegistry.setEnabled(id, enabled)}
          />
        </div>
      )}
    </div>
  );
}
