import type { ExtensionRecord, ExtensionUiStartupState } from "@/types/extensions";

export function advancedRecordsSignature(records: readonly ExtensionRecord[]): string {
  return JSON.stringify(records
    .filter((record) => record.kind === "local"
      && record.enabled && record.trusted
      && record.manifest.apiLevel === "advanced"
      && record.manifest.ui?.mode === "advanced"
      && Boolean(record.uiArtifact))
    .map((record) => ({
      id: record.manifest.id,
      apiVersion: record.manifest.ui?.apiVersion,
      manifestSha256: record.uiArtifact?.manifestSha256,
    }))
    .sort((left, right) => left.id.localeCompare(right.id)));
}

export function advancedStartupSignature(state: ExtensionUiStartupState | undefined): string {
  return JSON.stringify(state?.mode ?? null);
}
