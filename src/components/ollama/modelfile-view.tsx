import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { SystemPromptSettingsPanel } from "@/components/system-prompts/system-prompt-settings-panel";
import { systemPromptTierForModel } from "@/lib/system-prompt-tiers";
import "./ollama.css";
import "./modelfile-view.css";

interface ModelfileViewProps {
  modelName: string;
  parameters: { key: string; value: string }[] | null;
  parameterError: string | null;
  modelfile: string;
  onEditParameters: () => void;
}

export function ModelfileView({
  modelName,
  parameters,
  parameterError,
  modelfile,
  onEditParameters,
}: ModelfileViewProps) {
  const { t } = useTranslation();

  return (
    <>
      <SystemPromptSettingsPanel
        key={modelName}
        target={{ scope: "ollama", model: modelName }}
        warningKind="ollama"
        initialMode="agentic"
        initialTier={systemPromptTierForModel(modelName)}
      />

      <SettingsCard className="mfv-parameters-card">
        <ViewSection
          title={t("ollama.parameters")}
          editLabel={t("ollama.edit")}
          onEdit={onEditParameters}
          disabled={parameters === null}
        >
          {parameterError ? (
            <div className="mfv-parameter-error" role="alert">
              {parameterError}
            </div>
          ) : parameters?.length === 0 ? (
            <div className="mfv-no-params">
              {t("ollama.noParameters")}
            </div>
          ) : (
            <div className="mfv-params-list">
              {parameters?.map((p, i) => (
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
    </>
  );
}

function ViewSection({
  title, editLabel, onEdit, disabled = false, children,
}: {
  title: string; editLabel: string; onEdit: () => void;
  disabled?: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className="mfv-section">
      <div className="mfv-section-header">
        <span className="mfv-section-title">
          {title}
        </span>
        <button className="btn btn-sm btn-primary" onClick={onEdit} disabled={disabled}>
          {editLabel}
        </button>
      </div>
      {children}
    </div>
  );
}
