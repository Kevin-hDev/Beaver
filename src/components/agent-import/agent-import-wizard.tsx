import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowsClockwise, X } from "@/components/ui/icons";
import { useAgentImport } from "@/hooks/use-agent-import";
import { showToast } from "@/lib/toast-emitter";
import type { AgentSourceSelection } from "@/types/agent-import";
import { AgentSourceDetail } from "./agent-source-detail";
import { AgentSourceGrid } from "./agent-source-grid";
import "./agent-import.css";

interface AgentImportWizardProps {
  onContinue?: () => void;
  onBack?: () => void;
  onClose?: () => void;
}

export function AgentImportWizard({
  onContinue,
  onBack,
  onClose,
}: AgentImportWizardProps) {
  const { t } = useTranslation();
  const { sources, loading, saving, failed, scan, save } = useAgentImport();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [pending, setPending] = useState<AgentSourceSelection | null>(null);
  const [conflicts, setConflicts] = useState<string[]>([]);
  const selected = useMemo(
    () => sources.find((source) => source.id === selectedId) ?? null,
    [selectedId, sources],
  );

  const saveSelection = useCallback(async (selection: AgentSourceSelection) => {
    try {
      const result = await save(selection);
      if (!result.saved) {
        setPending(selection);
        setConflicts(result.conflicts);
        return;
      }
      setSelectedId(null);
      showToast(t("agentImport.messages.saved"), "success");
    } catch {
      showToast(t("errors.saveFailed"), "error");
    }
  }, [save, t]);

  const replaceDocuments = useCallback(async () => {
    if (!pending) return;
    try {
      const result = await save(pending, true);
      if (!result.saved) {
        setConflicts(result.conflicts);
        showToast(t("errors.saveFailed"), "error");
        return;
      }
      setPending(null);
      setConflicts([]);
      setSelectedId(null);
      showToast(t("agentImport.messages.saved"), "success");
    } catch {
      showToast(t("errors.saveFailed"), "error");
    }
  }, [pending, save, t]);

  if (selected) {
    return (
      <div className="aim-wizard">
        <AgentSourceDetail
          key={`${selected.id}-${selected.configured}-${selected.enabled}`}
          source={selected}
          saving={saving}
          conflicts={conflicts}
          onBack={() => {
            setPending(null);
            setConflicts([]);
            setSelectedId(null);
          }}
          onSave={(selection) => void saveSelection(selection)}
          onReplace={() => void replaceDocuments()}
          onCancelConflict={() => {
            setPending(null);
            setConflicts([]);
          }}
        />
      </div>
    );
  }

  return (
    <div className="aim-wizard">
      <div className="aim-heading">
        <div>
          <h1>{t("agentImport.title")}</h1>
          <p>{t("agentImport.description")}</p>
        </div>
        <div className="aim-heading-actions">
          <button
            type="button"
            className="aim-icon-btn"
            onClick={() => void scan()}
            aria-label={t("agentImport.actions.rescan")}
            disabled={loading}
          >
            <ArrowsClockwise size="var(--icon-md)" />
          </button>
          {onClose && (
            <button
              type="button"
              className="aim-icon-btn"
              onClick={onClose}
              aria-label={t("a11y.close")}
            >
              <X size="var(--icon-md)" />
            </button>
          )}
        </div>
      </div>

      <div className="aim-grid-scroll">
        {loading && <div className="aim-empty">{t("agentImport.loading")}</div>}
        {failed && !loading && (
          <div className="aim-empty">
            <span>{t("agentImport.messages.scanFailed")}</span>
            <button type="button" className="aim-btn aim-btn-secondary" onClick={() => void scan()}>
              {t("agentImport.actions.retry")}
            </button>
          </div>
        )}
        {!loading && !failed && (
          <AgentSourceGrid sources={sources} onSelect={setSelectedId} />
        )}
      </div>

      {onContinue && (
        <div className="aim-footer">
          <div className="aim-footer-actions">
            {onBack && (
              <button type="button" className="aim-btn aim-btn-secondary" onClick={onBack}>
                {t("onboarding.common.back")}
              </button>
            )}
            <button type="button" className="aim-btn aim-btn-primary" onClick={onContinue}>
              {t("onboarding.common.continue")}
            </button>
          </div>
          <span>{t("agentImport.onboardingLater")}</span>
        </div>
      )}
    </div>
  );
}
