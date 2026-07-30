import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { RegistryTag } from "@/types/agent";
import "./ollama.css";
import "./model-variants-list.css";

interface ModelVariantsListProps {
  familyName: string;
  onSelectVariant: (fullName: string) => void;
}

export function ModelVariantsList({ familyName, onSelectVariant }: ModelVariantsListProps) {
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

  const entries = useMemo(
    () => tags.map((tag) => {
      const fullName = `${familyName}:${tag.name}`;
      const local = localModels.find((model) => model.name === fullName);
      const hasUpdate = Boolean(local) && !local?.is_customized && local?.digest_short !== tag.digest_short;
      return {
        id: fullName,
        label: tag.name,
        description: [
          tag.size_gb ? `${tag.size_gb} GB` : "—",
          tag.context_length ? `${(tag.context_length / 1024).toFixed(0)}K ${t("ollama.ctx")}` : null,
        ].filter(Boolean).join(" · "),
        trailing: hasUpdate
          ? <span className="mvl-update-badge">{t("ollama.update")}</span>
          : local
            ? <Check size="var(--icon-sm)" className="mvl-installed-icon" />
            : undefined,
      };
    }),
    [familyName, localModels, t, tags],
  );

  if (loading) return <p className="settings-panel-description">{t("ollama.loadingVariants")}</p>;
  if (error) return <p className="mvl-error">{error}</p>;

  return (
    <SettingsEntryList
      entries={entries}
      emptyMessage={t("ollama.noVariants")}
      onSelect={onSelectVariant}
    />
  );
}
