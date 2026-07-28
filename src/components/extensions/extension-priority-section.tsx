import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretRight, Plus } from "@/components/ui/icons";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionIcon } from "./extension-icon";
import { ExtensionPriorityDialog } from "./extension-priority-dialog";
import { extensionDisplayName } from "./official-plugin-copy";
import "./extension-priority-section.css";

interface ExtensionPrioritySectionProps {
  records: ExtensionRecord[];
  selectedIds: string[];
  busy: boolean;
  onSave: (ids: string[]) => Promise<boolean>;
}

export function ExtensionPrioritySection({
  records,
  selectedIds,
  busy,
  onSave,
}: ExtensionPrioritySectionProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState(false);
  const byId = useMemo(
    () => new Map(records.map((record) => [record.manifest.id, record])),
    [records],
  );
  const selected = selectedIds.flatMap((id) => {
    const record = byId.get(id);
    return record ? [record] : [];
  });

  return (
    <section className="extpr-section">
      <button
        type="button"
        className="extpr-trigger"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
      >
        {open
          ? <CaretDown size="var(--icon-xs)" />
          : <CaretRight size="var(--icon-xs)" />}
        <span>{t("extensions.discovery.title")}</span>
        <small>{t("extensions.discovery.count", { count: selected.length })}</small>
      </button>
      {open && (
        <div className="extpr-content">
          <p>{t("extensions.discovery.description")}</p>
          {selected.length === 0 ? (
            <span className="extpr-empty">{t("extensions.discovery.empty")}</span>
          ) : (
            <div className="extpr-list">
              {selected.map((extension) => (
                <div className="extpr-row" key={extension.manifest.id}>
                  <span className="extpr-icon">
                    <ExtensionIcon extension={extension} />
                  </span>
                  <span>{extensionDisplayName(t, extension)}</span>
                </div>
              ))}
            </div>
          )}
          <button
            type="button"
            className="wk-btn-secondary extpr-add"
            disabled={busy}
            onClick={() => setEditing(true)}
          >
            <Plus size="var(--icon-sm)" weight="bold" />
            {t("extensions.discovery.add")}
          </button>
        </div>
      )}
      {editing && (
        <ExtensionPriorityDialog
          records={records}
          selectedIds={selectedIds}
          busy={busy}
          onCancel={() => setEditing(false)}
          onSave={async (ids) => {
            const saved = await onSave(ids);
            if (saved) setEditing(false);
          }}
        />
      )}
    </section>
  );
}
