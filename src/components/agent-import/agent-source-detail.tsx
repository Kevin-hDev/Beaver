import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretLeft } from "@/components/ui/icons";
import { InlineToast } from "@/components/ui/toast";
import {
  buildSourceSelection,
  createImportDraft,
  draftMatchesSource,
} from "@/lib/agent-import-selection";
import { cn } from "@/lib/utils";
import type {
  AgentSourceSelection,
  AgentSourceSummary,
} from "@/types/agent-import";
import { AgentImportListSection } from "./agent-import-list-section";
import { AgentSourceLogo } from "./agent-source-logo";
import "./agent-import-detail.css";

interface AgentSourceDetailProps {
  source: AgentSourceSummary;
  saving: boolean;
  conflicts: string[];
  onBack: () => void;
  onSave: (selection: AgentSourceSelection) => void;
  onReplace: () => void;
  onCancelConflict: () => void;
}

export function AgentSourceDetail({
  source,
  saving,
  conflicts,
  onBack,
  onSave,
  onReplace,
  onCancelConflict,
}: AgentSourceDetailProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(() => createImportDraft(source));
  const draftChanged = !draftMatchesSource(source, draft);
  const importedDocumentIds = source.configured
    ? createImportDraft(source).documentIds
    : new Set<string>();
  const hasDocumentUpdate = source.documents.some(
    (item) => item.selected && item.updateAvailable,
  );
  const showPrimaryAction = !source.configured
    || !source.enabled
    || draftChanged
    || hasDocumentUpdate;
  const statusLabel = source.configured
    ? source.enabled
      ? t("agentImport.card.active")
      : t("agentImport.card.disabled")
    : t(`agentImport.status.${source.status}`);

  return (
    <div className="aim-detail">
      <button type="button" className="aim-back" onClick={onBack}>
        <CaretLeft size="var(--icon-sm)" />
        {t("agentImport.actions.back")}
      </button>

      <div className="aim-detail-heading">
        <div className="aim-detail-brand">
          <AgentSourceLogo
            sourceId={source.id}
            displayName={source.displayName}
            variant="detail"
          />
          <p>{t("agentImport.detail.description")}</p>
        </div>
        <span className={cn(
          "aim-status-badge",
          source.configured
            ? source.enabled ? "is-active" : "is-disabled"
            : `aim-status-${source.status}`,
        )}>
          {statusLabel}
        </span>
      </div>

      {source.partial && (
        <InlineToast type="warning">
          {t("agentImport.detail.partial")}
        </InlineToast>
      )}

      {conflicts.length > 0 && (
        <div className="aim-conflict" role="alert">
          <strong>{t("agentImport.conflict.title")}</strong>
          <span>
            {t("agentImport.conflict.description", {
              names: conflicts.join(", "),
            })}
          </span>
          <div>
            <button type="button" onClick={onCancelConflict}>
              {t("agentImport.conflict.keep")}
            </button>
            <button type="button" onClick={onReplace} disabled={saving}>
              {t("agentImport.conflict.replace")}
            </button>
          </div>
        </div>
      )}

      <div className="aim-detail-scroll">
        <AgentImportListSection
          title={t("agentImport.sections.documents")}
          items={source.documents}
          selectedIds={draft.documentIds}
          lockedIds={importedDocumentIds}
          onChange={(documentIds) => setDraft((current) => ({ ...current, documentIds }))}
        />
        <AgentImportListSection
          title={t("agentImport.sections.rules")}
          items={source.rules}
          selectedIds={draft.ruleIds}
          onChange={(ruleIds) => setDraft((current) => ({ ...current, ruleIds }))}
          bulkActions
        />
        <AgentImportListSection
          title={t("agentImport.sections.skills")}
          items={source.skills}
          selectedIds={draft.skillIds}
          onChange={(skillIds) => setDraft((current) => ({ ...current, skillIds }))}
          searchable
          bulkActions
        />
        {source.documents.length + source.rules.length + source.skills.length === 0 && (
          <div className="aim-empty">{t("agentImport.detail.empty")}</div>
        )}
      </div>

      <div className="aim-detail-actions">
        {source.configured && source.enabled && (
          <button
            type="button"
            className="aim-btn aim-btn-secondary"
            onClick={() => onSave(buildSourceSelection(source, draft, false))}
            disabled={saving}
          >
            {t("agentImport.actions.disable")}
          </button>
        )}
        {showPrimaryAction && (
          <button
            type="button"
            className="aim-btn aim-btn-primary"
            onClick={() => onSave(buildSourceSelection(source, draft))}
            disabled={saving || source.status === "missing"}
          >
            {saving
              ? t("agentImport.actions.saving")
              : !source.configured
                ? t("agentImport.actions.confirmSource")
                : source.enabled
                  ? t("agentImport.actions.confirmChanges")
                  : t("agentImport.actions.reactivate")}
          </button>
        )}
      </div>
    </div>
  );
}
