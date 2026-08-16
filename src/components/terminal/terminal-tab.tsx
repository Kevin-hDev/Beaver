import { useTranslation } from "react-i18next";
import { X as XIcon } from "@/components/ui/icons";
import { TerminalIcon } from "@/components/ui/chat-header-icons";
import type { TerminalTab } from "@/hooks/use-terminal";
import type { DragItemProps } from "@/hooks/use-drag-reorder";

interface TerminalTabItemProps {
  tab: TerminalTab;
  isActive: boolean;
  isEditing: boolean;
  dragProps: DragItemProps;
  onSelect: () => void;
  onClose: () => void;
  onRename: (label: string) => void;
  onEditStart: () => void;
  onEditEnd: () => void;
  onPointerDown: (e: React.PointerEvent) => void;
}

export function TerminalTabItem({
  tab,
  isActive,
  isEditing,
  dragProps,
  onSelect,
  onClose,
  onRename,
  onEditStart,
  onEditEnd,
  onPointerDown,
}: TerminalTabItemProps) {
  const { t } = useTranslation();

  return (
    <div
      {...dragProps}
      className={isActive ? "terminal-tab-item active" : "terminal-tab-item"}
      role="button"
      tabIndex={0}
      title={tab.label}
      /* Un glissement se termine par un clic que le navigateur envoie quand
         même : le filtre est posé par la barre, qui sait s'il y a eu geste. */
      onClick={onSelect}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onSelect(); }}
      onPointerDown={onPointerDown}
      onDoubleClick={onEditStart}
    >
      <span className="terminal-tab-icon">
        {tab.hasActivity && !isActive
          ? <span className="terminal-tab-dot" title={t("terminal.activity")} />
          : <TerminalIcon size="var(--terminal-tab-icon-size)" />}
      </span>
      {isEditing ? (
        <input
          autoFocus
          className="terminal-tab-rename"
          defaultValue={tab.label}
          onFocus={(e) => e.target.select()}
          onBlur={(e) => { onRename(e.target.value); onEditEnd(); }}
          onKeyDown={(e) => {
            if (e.code === "Enter" || e.code === "NumpadEnter") {
              onRename(e.currentTarget.value);
              onEditEnd();
            }
            if (e.code === "Escape") onEditEnd();
          }}
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <span className="terminal-tab-label">{tab.label}</span>
      )}
      <button
        className="terminal-tab-close"
        aria-label={t("terminal.closeTab")}
        title={t("terminal.closeTab")}
        onClick={(e) => { e.stopPropagation(); onClose(); }}
        onPointerDown={(e) => e.stopPropagation()}
      >
        <XIcon size="var(--terminal-tab-close-icon-size)" />
      </button>
    </div>
  );
}
