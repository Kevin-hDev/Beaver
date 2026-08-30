import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsSelect } from "@/components/settings/settings-select";
import {
  EditableRowActions,
  useEditableRowActions,
} from "@/components/ui/editable-row-actions";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import { CompressionProfileDialog } from "./compression-profile-dialog";
import { offerCompressionProfileUndo } from "./compression-profile-undo";

interface CompressionProfileBarProps {
  controller: CompressionProfilesController;
  onInteractionChange: (active: boolean) => void;
}

export function CompressionProfileBar({
  controller,
  onInteractionChange,
}: CompressionProfileBarProps) {
  const { t } = useTranslation();
  const [creating, setCreating] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const newButtonRef = useRef<HTMLButtonElement>(null);
  const active = controller.view?.profiles.find(
    (profile) => profile.id === controller.view?.global_profile_id,
  );
  const actions = useEditableRowActions({
    rootRef,
    value: active?.name ?? "",
    onRename: async (name) => {
      if (!active || !await controller.rename(active.id, name)) throw new Error("rename_failed");
    },
    onDelete: async () => {
      if (!active) throw new Error("delete_failed");
      const deletedName = active.name;
      const result = await controller.deleteProfile(active.id);
      if (!result) throw new Error("delete_failed");
      offerCompressionProfileUndo(
        t("settings.advanced.compressionDeleted", { name: deletedName }),
        t("settings.advanced.compressionUndo"),
        result.undo_expires_in_ms,
        () => { void controller.undoDelete(result.undo_token); },
      );
    },
    onInteractionChange,
  });
  const options = (controller.view?.profiles ?? []).map((profile) => ({
    value: profile.id,
    label: profile.name,
  }));

  if (!controller.view || !active) return <div className="cpb-loading" aria-busy="true">—</div>;
  const builtIn = active.id === "beaver";

  const resetBeaver = async () => {
    const result = await controller.resetBeaver();
    if (!result) return;
    offerCompressionProfileUndo(
      t("settings.advanced.compressionResetDone"),
      t("settings.advanced.compressionUndo"),
      result.undo_expires_in_ms,
      () => { void controller.undoDelete(result.undo_token); },
    );
  };

  return (
    <div ref={rootRef} className="cpb-bar">
      <span className="cpb-label">{t("settings.advanced.compressionProfileLabel")}</span>
      {actions.editing ? (
        <input
          className="field cpb-rename"
          autoFocus
          value={actions.draft}
          aria-label={t("settings.advanced.compressionProfileName")}
          onChange={(event) => actions.setDraft(event.target.value)}
        />
      ) : (
        <SettingsSelect
          options={options}
          value={active.id}
          disabled={controller.busy}
          fitLongestOption
          onChange={(profileId) => { void controller.selectGlobal(profileId); }}
        />
      )}
      <div className="cpb-actions">
        <button
          ref={newButtonRef}
          type="button"
          className="btn btn-sm btn-secondary"
          disabled={controller.busy || actions.editing}
          onClick={() => { setCreating(true); onInteractionChange(true); }}
        >
          {t("settings.advanced.compressionNewProfile")}
        </button>
        <EditableRowActions
          controller={actions}
          disabled={builtIn || controller.busy}
          renameLabel={t("settings.advanced.compressionRename")}
          deleteLabel={t("settings.advanced.compressionDelete")}
          confirmLabel={t("settings.advanced.compressionDelete")}
          cancelLabel={t("settings.advanced.compressionCancel")}
          confirmationMessage={t("settings.advanced.compressionDeleteConfirm", { name: active.name })}
          confirmationPlacement="below"
        />
        {builtIn && (
          <button type="button" className="btn btn-sm btn-ghost" disabled={controller.busy} onClick={() => { void resetBeaver(); }}>
            {t("settings.advanced.compressionReset")}
          </button>
        )}
      </div>
      {creating && (
        <CompressionProfileDialog
          sourceName={active.name}
          existingNames={controller.view.profiles.map((profile) => profile.name)}
          onCancel={() => {
            setCreating(false);
            onInteractionChange(false);
            requestAnimationFrame(() => newButtonRef.current?.focus());
          }}
          onCreate={(name) => controller.create(active.id, name)}
        />
      )}
    </div>
  );
}
