import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { ModelInstallButton } from "./model-install-button";
import { TranslationControls } from "./translation-controls";
import { buildSpecRows } from "./model-profile-specs";
import { SettingsCard } from "@/components/settings/settings-card";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import type {
  RegistryModelDetails,
  RegistryTag,
  ModelInfo,
} from "@/types/agent";
import "./ollama.css";
import "./ollama-details.css";
import "./model-profile.css";

interface ModelProfileProps {
  familyName: string;
  variantFullName: string | null;
}

export function ModelProfile({
  familyName,
  variantFullName,
}: ModelProfileProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();
  const [details, setDetails] = useState<RegistryModelDetails | null>(null);
  const [tags, setTags] = useState<RegistryTag[]>([]);
  const [localInfo, setLocalInfo] = useState<ModelInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [translated, setTranslated] = useState<string | null>(null);
  const [translatedLang, setTranslatedLang] = useState<string | null>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setDetails(null);
    setLocalInfo(null);
    setTranslated(null);
    setTranslatedLang(null);
    Promise.all([
      invoke<RegistryModelDetails>("get_registry_model_details", {
        name: familyName,
      }),
      invoke<RegistryTag[]>("list_registry_tags", { name: familyName }),
    ])
      .then(([d, ts]) => {
        setDetails(d);
        setTags(ts);
      })
      .catch((e) => console.warn("[ollama] model profile:", e))
      .finally(() => setLoading(false));
  }, [familyName]);

  useEffect(() => {
    let cancelled = false;
    const resetLocalInfo = () => {
      queueMicrotask(() => {
        if (!cancelled) setLocalInfo(null);
      });
    };
    if (!variantFullName) {
      resetLocalInfo();
      return () => {
        cancelled = true;
      };
    }
    const installed = localModels.find((m) => m.name === variantFullName);
    if (!installed) {
      resetLocalInfo();
      return () => {
        cancelled = true;
      };
    }
    invoke<ModelInfo>("show_ollama_model", { name: variantFullName })
      .then((info) => {
        if (!cancelled) setLocalInfo(info);
      })
      .catch(() => resetLocalInfo());
    return () => {
      cancelled = true;
    };
  }, [variantFullName, localModels]);

  const currentTag = useMemo(() => {
    if (!variantFullName) return null;
    const tagName = variantFullName.split(":")[1];
    return tags.find((tg) => tg.name === tagName) ?? null;
  }, [variantFullName, tags]);

  const displayName = variantFullName ?? familyName;
  const installedLocal = variantFullName
    ? localModels.find((m) => m.name === variantFullName)
    : null;
  const hasUpdate = Boolean(
    installedLocal &&
    !installedLocal.is_customized &&
    currentTag &&
    installedLocal.digest_short !== currentTag.digest_short,
  );

  if (loading) {
    return <div className="mp-loading">{t("ollama.loadingProfile")}</div>;
  }

  const rows = buildSpecRows(t, details, currentTag, localInfo);

  return (
    <div className="mp-root">
      <div className="mp-inner">
        <div className="mp-header">
          <h2 className="mp-title">{displayName}</h2>
          {variantFullName && (
            <ModelInstallButton
              fullName={variantFullName}
              isInstalled={Boolean(installedLocal)}
              hasUpdate={hasUpdate}
              sizeGb={currentTag?.size_gb ?? undefined}
            />
          )}
        </div>

        {details?.description_short && (
          <div className="mp-description">{details.description_short}</div>
        )}

        <SettingsCard>
          {rows.map((r, i) => (
            <div
              key={i}
              className={`mp-spec-row${i < rows.length - 1 ? " mp-spec-row-border" : ""}`}
            >
              <span className="mp-spec-label">{r.label}</span>
              <span className={r.mono ? "mp-spec-value-mono" : "mp-spec-value"}>
                {r.value}
              </span>
            </div>
          ))}
        </SettingsCard>

        {details?.description_long_markdown && (
          <div className="mp-translation-wrapper">
            <TranslationControls
              modelName={familyName}
              originalText={details.description_long_markdown}
              currentLang={translatedLang}
              onChange={(lang, text) => {
                setTranslatedLang(lang);
                setTranslated(text);
              }}
            />
          </div>
        )}
      </div>

      {details?.description_long_markdown && (
        <div className="ollama-readme mp-readme">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeRaw, rehypeSanitize]}
          >
            {translated ?? details.description_long_markdown}
          </ReactMarkdown>
        </div>
      )}
    </div>
  );
}
