import type {
  RegistryModelDetails,
  RegistryTag,
  ModelInfo,
} from "@/types/agent";

interface SpecRow {
  label: string;
  value: string;
  mono?: boolean;
}

export function buildSpecRows(
  t: (k: string) => string,
  details: RegistryModelDetails | null,
  tag: RegistryTag | null,
  info: ModelInfo | null,
): SpecRow[] {
  const rows: SpecRow[] = [];
  if (details?.capabilities?.length) {
    rows.push({
      label: t("ollama.capabilities"),
      value: details.capabilities.join(", "),
    });
  }
  if (tag?.size_gb) {
    rows.push({ label: t("ollama.fileSize"), value: `${tag.size_gb} GB` });
  }
  if (info?.parameter_size) {
    rows.push({ label: t("ollama.paramsLabel"), value: info.parameter_size });
  }
  const ctx = tag?.context_length ?? details?.context_length;
  if (ctx) {
    rows.push({
      label: t("ollama.context"),
      value: `${(ctx / 1024).toFixed(0)}${t("ollama.contextTokens")}`,
    });
  }
  if (info?.quantization) {
    rows.push({ label: t("ollama.quantization"), value: info.quantization });
  }
  if (info?.architecture) {
    rows.push({ label: t("ollama.architecture"), value: info.architecture });
  }
  if (info) {
    rows.push({
      label: t("ollama.moe"),
      value: info.is_moe ? t("ollama.yes") : t("ollama.no"),
    });
  }
  if (tag?.digest_short) {
    rows.push({
      label: t("ollama.digest"),
      value: tag.digest_short,
      mono: true,
    });
  }
  return rows;
}
