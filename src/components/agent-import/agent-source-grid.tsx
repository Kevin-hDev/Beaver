import { useTranslation } from "react-i18next";
import { CaretRight, CheckCircle2 } from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import type { AgentSourceSummary } from "@/types/agent-import";
import { AgentSourceLogo } from "./agent-source-logo";
import "./agent-source-grid.css";

interface AgentSourceGridProps {
  sources: AgentSourceSummary[];
  onSelect: (sourceId: string) => void;
}

export function AgentSourceGrid({ sources, onSelect }: AgentSourceGridProps) {
  const { t } = useTranslation();

  return (
    <div className="aim-grid">
      {sources.map((source) => {
        const activeCount = [
          ...source.documents,
          ...source.rules,
          ...source.skills,
        ].filter((item) => item.selected).length;
        return (
          <button
            key={source.id}
            type="button"
            className={cn(
              "aim-source-card",
              `aim-status-${source.status}`,
              source.configured && "is-configured",
            )}
            onClick={() => onSelect(source.id)}
          >
            <AgentSourceLogo
              sourceId={source.id}
              displayName={source.displayName}
              variant="card"
            />
            <span className="aim-source-copy">
              <span className="aim-source-name-row">
                <span className="aim-source-name">{source.displayName}</span>
                {source.configured && (
                  <span className={cn(
                    "aim-source-state",
                    !source.enabled && "is-disabled",
                  )}>
                    {source.enabled && (
                      <CheckCircle2 size="var(--icon-sm)" weight="fill" />
                    )}
                    {t(source.enabled
                      ? "agentImport.card.migrated"
                      : "agentImport.card.disabled")}
                  </span>
                )}
              </span>
              <span className="aim-source-status">
                {t(`agentImport.status.${source.status}`)}
              </span>
              <span className="aim-source-count">
                {t("agentImport.card.counts", {
                  documents: source.documents.length,
                  rules: source.rules.length,
                  skills: source.skills.length,
                })}
              </span>
              {source.configured && source.enabled && (
                <span className="aim-source-active">
                  {t("agentImport.card.activeCount", { count: activeCount })}
                </span>
              )}
            </span>
            <CaretRight className="aim-source-caret" size="var(--icon-md)" />
          </button>
        );
      })}
    </div>
  );
}
