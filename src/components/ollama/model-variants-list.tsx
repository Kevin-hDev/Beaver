import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type { RegistryTag, OllamaModel } from "@/types/agent";
import "./ollama.css";
import "./model-variants-list.css";

interface ModelVariantsListProps {
  familyName: string;
  selectedVariant: string | null;
  onSelectVariant: (fullName: string) => void;
  onBack: () => void;
}

export function ModelVariantsList({
  familyName, selectedVariant, onSelectVariant, onBack,
}: ModelVariantsListProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();
  const [tags, setTags] = useState<RegistryTag[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setError(null);
    invoke<RegistryTag[]>("list_registry_tags", { name: familyName })
      .then((list) => setTags(list))
      .catch(() => setError(t("errors.operationFailed")))
      .finally(() => setLoading(false));
  }, [familyName, t]);

  const findLocal = (tagName: string): OllamaModel | undefined => {
    const fullName = `${familyName}:${tagName}`;
    return localModels.find((m) => m.name === fullName);
  };

  return (
    <div className="mvl-root">
      <button
        onClick={onBack}
        className="ollama-btn mvl-back-btn"
      >
        ← {familyName}
      </button>

      {loading && (
        <div className="mvl-loading">
          {t("ollama.loadingVariants")}
        </div>
      )}

      {error && (
        <div className="mvl-error">
          {error}
        </div>
      )}

      <div className="mvl-list">
        {tags.map((tag) => {
          const local = findLocal(tag.name);
          const installed = Boolean(local);
          const hasUpdate = installed && !local?.is_customized && local?.digest_short !== tag.digest_short;
          const fullName = `${familyName}:${tag.name}`;
          const isActive = selectedVariant ? selectedVariant === fullName : false;

          return (
            <div
              key={tag.name}
              className={`ollama-model-item mvl-item-row ${isActive ? "active" : ""}`}
              role="button"
              tabIndex={0}
              onClick={() => onSelectVariant(fullName)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelectVariant(fullName); }}
            >
              <div className="mvl-item-content">
                <div className="mvl-item-name">
                  {tag.name}
                </div>
                <div className="mvl-item-meta">
                  {tag.size_gb ? `${tag.size_gb} GB` : "—"}
                  {tag.context_length ? ` · ${(tag.context_length / 1024).toFixed(0)}K ${t("ollama.ctx")}` : ""}
                </div>
              </div>
              {hasUpdate && (
                <span className="mvl-update-badge">
                  {t("ollama.update")}
                </span>
              )}
              {installed && !hasUpdate && (
                <Check size="var(--icon-sm)" className="mvl-installed-icon" />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
