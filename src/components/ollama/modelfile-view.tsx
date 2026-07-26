import { useTranslation } from "react-i18next";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { SettingsCard } from "@/components/settings/settings-card";
import "./ollama.css";
import "./modelfile-view.css";

interface ModelfileViewProps {
  modelName: string;
  systemPrompt: string;
  parameters: { key: string; value: string }[];
  modelfile: string;
  deleting: boolean;
  onDelete: () => void;
  onEditSystem: () => void;
  onEditParameters: () => void;
  onEditModelfile: () => void;
}

export function ModelfileView({
  modelName,
  systemPrompt,
  parameters,
  modelfile,
  deleting,
  onDelete,
  onEditSystem,
  onEditParameters,
  onEditModelfile,
}: ModelfileViewProps) {
  const { t } = useTranslation();

  return (
    <div className="mfv-scroll">
      <div className="mfv-inner">
        <div className="mfv-header">
          <h2 className="mfv-title">
            {modelName}
          </h2>
          <div className="mfv-actions">
            <ConfirmButton
              className="ollama-btn"
              label={t("ollama.remove")}
              confirmLabel={t("settings.confirm.deleteModel")}
              onConfirm={onDelete}
              disabled={deleting}
            />
            <button className="ollama-btn" onClick={onEditModelfile}>
              {t("ollama.editModelfile")}
            </button>
          </div>
        </div>

        <SettingsCard>
          <ViewSection title={t("ollama.systemPrompt")} editLabel={t("ollama.edit")} onEdit={onEditSystem}>
            <div className={`mfv-system-prompt ${systemPrompt ? "mfv-system-prompt-filled" : "mfv-system-prompt-empty"}`}>
              {systemPrompt || t("ollama.noSystemPrompt")}
            </div>
          </ViewSection>

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
        <button className="ollama-btn ollama-btn-primary" onClick={onEdit}>
          {editLabel}
        </button>
      </div>
      {children}
    </div>
  );
}
