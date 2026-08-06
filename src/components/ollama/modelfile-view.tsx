import { useTranslation } from "react-i18next";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { SettingsCard } from "@/components/settings/settings-card";
import { SystemPromptSettingsPanel } from "@/components/system-prompts/system-prompt-settings-panel";
import { systemPromptTierForModel } from "@/lib/system-prompt-tiers";
import "./ollama.css";
import "./modelfile-view.css";

interface ModelfileViewProps {
  modelName: string;
  parameters: { key: string; value: string }[];
  modelfile: string;
  deleting: boolean;
  onDelete: () => void;
  onEditParameters: () => void;
  onEditModelfile: () => void;
}

export function ModelfileView({
  modelName,
  parameters,
  modelfile,
  deleting,
  onDelete,
  onEditParameters,
  onEditModelfile,
}: ModelfileViewProps) {
  const { t } = useTranslation();

  return (
    <div className="mfv-scroll">
      <div className="mfv-inner">
        <SystemPromptSettingsPanel
          key={modelName}
          target={{ scope: "ollama", model: modelName }}
          warningKind="ollama"
          initialMode="agentic"
          initialTier={systemPromptTierForModel(modelName)}
          selectorHeader={<h2 className="mfv-title">{modelName}</h2>}
          selectorActions={(
            <div className="mfv-actions">
              <ConfirmButton
                className="btn btn-sm btn-secondary"
                label={t("ollama.remove")}
                confirmLabel={t("settings.confirm.deleteModel")}
                onConfirm={onDelete}
                disabled={deleting}
              />
              <button className="btn btn-sm btn-secondary" onClick={onEditModelfile}>
                {t("ollama.editModelfile")}
              </button>
            </div>
          )}
        />

        <SettingsCard className="mfv-parameters-card">
          <ViewSection title={t("ollama.parameters")} editLabel={t("ollama.edit")} onEdit={onEditParameters} last>
            {parameters.length === 0 ? (
              <div className="mfv-no-params">
                {t("ollama.noParameters")}
              </div>
            ) : (
              <div className="mfv-params-list">
                {parameters.map((p, i) => (
                  <div key={i} className="mfv-param-row">
                    <span className="mfv-param-key">{p.key}</span>
                    <span className="mfv-param-value">{p.value}</span>
                  </div>
                ))}
              </div>
            )}
          </ViewSection>
        </SettingsCard>

        <pre className="mf-raw-block">
          {modelfile}
        </pre>
      </div>
    </div>
  );
}

function ViewSection({
  title, editLabel, onEdit, children, last,
}: {
  title: string; editLabel: string; onEdit: () => void;
  children: React.ReactNode; last?: boolean;
}) {
  return (
    <div className={`mfv-section ${last ? "" : "mfv-section-border"}`}>
      <div className="mfv-section-header">
        <span className="mfv-section-title">
          {title}
        </span>
        <button className="btn btn-sm btn-primary" onClick={onEdit}>
          {editLabel}
        </button>
      </div>
      {children}
    </div>
  );
}
