import { useCallback, useEffect, useState } from "react";
import { Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import "./editable-row-actions.css";

interface EditableRowOptions {
  rootRef: React.RefObject<HTMLDivElement | null>;
  value: string;
  onRename: (name: string) => Promise<void>;
  onDelete: () => Promise<void>;
  onInteractionChange?: (active: boolean) => void;
}

export interface EditableRowController {
  editing: boolean;
  confirmingDelete: boolean;
  draft: string;
  setDraft(value: string): void;
  startRename(): void;
  startDelete(): void;
  cancel(): void;
  commitRename(): Promise<void>;
  confirmDelete(): Promise<void>;
}

export function useEditableRowActions({
  rootRef,
  value,
  onRename,
  onDelete,
  onInteractionChange,
}: EditableRowOptions): EditableRowController {
  const [editing, setEditing] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [draft, setDraft] = useState(value);

  const cancel = useCallback(() => {
    setEditing(false);
    setConfirmingDelete(false);
    setDraft(value);
    onInteractionChange?.(false);
  }, [onInteractionChange, value]);

  const commitRename = useCallback(async () => {
    const trimmed = draft.trim();
    try {
      if (trimmed && trimmed !== value) await onRename(trimmed);
      setEditing(false);
      setDraft(trimmed || value);
      onInteractionChange?.(false);
    } catch {
      // The owner already surfaces a generic error; keep the editor open.
    }
  }, [draft, onInteractionChange, onRename, value]);

  const confirmDelete = useCallback(async () => {
    try {
      await onDelete();
      setConfirmingDelete(false);
      onInteractionChange?.(false);
    } catch {
      // The owner already surfaces a generic error; keep the confirmation open.
    }
  }, [onDelete, onInteractionChange]);

  useEffect(() => {
    if (!editing && !confirmingDelete) return;
    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) cancel();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        cancel();
      } else if (event.key === "Enter") {
        event.preventDefault();
        if (editing) void commitRename();
        if (confirmingDelete) void confirmDelete();
      }
    };
    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [cancel, commitRename, confirmDelete, confirmingDelete, editing, rootRef]);

  return {
    editing,
    confirmingDelete,
    draft,
    setDraft,
    startRename: () => {
      setConfirmingDelete(false);
      setDraft(value);
      setEditing(true);
      onInteractionChange?.(true);
    },
    startDelete: () => {
      setEditing(false);
      setConfirmingDelete(true);
      onInteractionChange?.(true);
    },
    cancel,
    commitRename,
    confirmDelete,
  };
}

interface EditableRowActionsProps {
  controller: EditableRowController;
  renameLabel: string;
  deleteLabel: string;
  confirmLabel: string;
  cancelLabel: string;
  confirmationMessage?: string;
  confirmationPlacement?: "above" | "below" | "side";
  disabled?: boolean;
}

export function EditableRowActions({
  controller,
  renameLabel,
  deleteLabel,
  confirmLabel,
  cancelLabel,
  confirmationMessage,
  confirmationPlacement = "above",
  disabled = false,
}: EditableRowActionsProps) {
  return (
    <div className="era-actions">
      {controller.confirmingDelete && (
        <div
          className={`era-confirm era-confirm-${confirmationPlacement} relief`}
          role="dialog"
          aria-label={deleteLabel}
        >
          {confirmationMessage && <p className="era-confirm-message">{confirmationMessage}</p>}
          <button type="button" className="btn btn-sm btn-primary" onClick={(event) => { event.stopPropagation(); void controller.confirmDelete(); }}>
            {confirmLabel}
          </button>
          <button type="button" className="btn btn-sm btn-secondary" onClick={(event) => { event.stopPropagation(); controller.cancel(); }}>
            {cancelLabel}
          </button>
        </div>
      )}
      <Tooltip label={renameLabel}>
        <button
          type="button"
          className="icon-btn icon-btn-secondary"
          aria-label={renameLabel}
          disabled={disabled}
          onClick={(event) => { event.stopPropagation(); controller.startRename(); }}
        >
          <Pencil size="var(--icon-15)" />
        </button>
      </Tooltip>
      <Tooltip label={deleteLabel}>
        <button
          type="button"
          className="icon-btn icon-btn-secondary icon-btn-destructive"
          aria-label={deleteLabel}
          disabled={disabled}
          onClick={(event) => { event.stopPropagation(); controller.startDelete(); }}
        >
          <Trash size="var(--icon-15)" />
        </button>
      </Tooltip>
    </div>
  );
}
