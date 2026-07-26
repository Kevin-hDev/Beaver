import { useTranslation } from "react-i18next";
import { SettingsCard } from "./settings-card";
import type { RegistryModelInfo } from "./llm-types";
import "./llm-explorer.css";

interface LlmModelListProps {
  models: RegistryModelInfo[];
  title: string;
  onSelect: (model: RegistryModelInfo) => void;
  onBack?: () => void;
}

export function LlmModelList({ models, title, onSelect, onBack }: LlmModelListProps) {
  const { t } = useTranslation();

  return (
    <div>
      {onBack && (
        <button
          onClick={onBack}
          style={{
            background: "none", border: "none", color: "var(--ink-muted)",
            cursor: "pointer", fontSize: "var(--text-xs)", marginBottom: 8,
            padding: 0,
          }}
        >
          &larr; {t("settings.llm.back")}
        </button>
      )}
      <h3 style={{
        fontSize: "var(--text-lg)", fontWeight: 600,
        color: "var(--ink)", marginBottom: 12,
      }}>
        {title} <span style={{ color: "var(--ink-faint)", fontWeight: 400 }}>({models.length})</span>
      </h3>

      <SettingsCard>
        {models.map((m, i) => (
          <div
            key={m.key}
            className="llm-model-row"
            role="button"
            tabIndex={0}
            onClick={() => onSelect(m)}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect(m); }}
            style={{ borderBottom: i < models.length - 1 ? "1px solid var(--edge)" : "none" }}
          >
            <div className="llm-model-row-name">{m.key}</div>
            <div className="llm-model-row-meta">
              <span>{m.provider}</span>
              {m.input_cost_per_mtok != null && (
                <span>${m.input_cost_per_mtok.toFixed(2)}/M</span>
              )}
              {m.max_input_tokens && (
                <span>{(m.max_input_tokens / 1000).toFixed(0)}K ctx</span>
              )}
            </div>
          </div>
        ))}
      </SettingsCard>
    </div>
  );
}
