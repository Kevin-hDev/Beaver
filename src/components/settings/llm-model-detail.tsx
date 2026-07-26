import { useTranslation } from "react-i18next";
import { SettingsCard } from "./settings-card";
import type { RegistryModelInfo } from "./llm-types";

interface LlmModelDetailProps {
  model: RegistryModelInfo;
  onBack: () => void;
}

export function LlmModelDetail({ model, onBack }: LlmModelDetailProps) {
  const { t } = useTranslation();
  const rows = buildRows(t, model);
  const caps = buildCaps(t, model);

  return (
    <div>
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
      <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", margin: "0 0 6px" }}>
        {model.key}
      </h2>
      <div style={{ fontSize: "var(--text-xs)", color: "var(--ink-muted)", marginBottom: 20 }}>
        {model.provider} &middot; {model.mode}
      </div>

      <SettingsCard>
        {rows.map((r, i) => (
          <DetailRow key={i} label={r.label} value={r.value} last={i >= rows.length - 1} />
        ))}
      </SettingsCard>

      {caps.length > 0 && (
        <div style={{ marginTop: 16 }}>
          <SettingsCard>
            {caps.map((c, i) => (
              <DetailRow key={i} label={c.label} value={c.value} last={i >= caps.length - 1} />
            ))}
          </SettingsCard>
        </div>
      )}
    </div>
  );
}

function DetailRow({ label, value, last }: { label: string; value: string; last: boolean }) {
  return (
    <div style={{
      display: "flex", justifyContent: "space-between", alignItems: "center",
      padding: "10px 20px", borderBottom: last ? "none" : "1px solid var(--edge)",
      fontSize: "var(--text-sm)",
    }}>
      <span style={{ color: "var(--ink-muted)", fontSize: "var(--text-xs)" }}>{label}</span>
      <span style={{ color: "var(--ink)", fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)" }}>{value}</span>
    </div>
  );
}

function buildRows(t: (k: string) => string, m: RegistryModelInfo) {
  const rows: { label: string; value: string }[] = [];
  if (m.max_input_tokens) rows.push({ label: t("settings.llm.maxInput"), value: `${(m.max_input_tokens / 1000).toFixed(0)}K` });
  if (m.max_output_tokens) rows.push({ label: t("settings.llm.maxOutput"), value: `${(m.max_output_tokens / 1000).toFixed(0)}K` });
  if (m.input_cost_per_mtok != null) rows.push({ label: t("settings.llm.inputCost"), value: `$${m.input_cost_per_mtok.toFixed(2)}/M` });
  if (m.output_cost_per_mtok != null) rows.push({ label: t("settings.llm.outputCost"), value: `$${m.output_cost_per_mtok.toFixed(2)}/M` });
  return rows;
}

function buildCaps(t: (k: string) => string, m: RegistryModelInfo) {
  const yes = t("settings.llm.yes");
  const no = t("settings.llm.no");
  return [
    { label: t("settings.llm.vision"), value: m.supports_vision ? yes : no },
    { label: t("settings.llm.tools"), value: m.supports_function_calling ? yes : no },
    { label: t("settings.llm.reasoning"), value: m.supports_reasoning ? yes : no },
    { label: t("settings.llm.caching"), value: m.supports_prompt_caching ? yes : no },
    { label: t("settings.llm.webSearch"), value: m.supports_web_search ? yes : no },
  ];
}
