import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { ShieldWarning } from "@/components/ui/icons";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";
import { formatExtensionDate } from "@/lib/extension-date";
import { useOptionalStandardCatalog } from "@/features/extension-ui/standard/catalog-context";
import { localizedText } from "@/features/extension-ui/standard/localized-text";
import type { StandardCatalogEntry } from "@/features/extension-ui/standard/types";
import { UI_DIAGNOSTIC_CODES } from "@/types/extension-ui-contract.generated";
import type { ExtensionDiagnostic, ExtensionRecord } from "@/types/extensions";

export function ExtensionUiDetail({
  extension,
  diagnostics,
  busy,
  onRetry,
}: {
  extension: ExtensionRecord;
  diagnostics: ExtensionDiagnostic[];
  busy: boolean;
  onRetry: () => void;
}) {
  const { t, i18n } = useTranslation();
  const catalog = useOptionalStandardCatalog();
  const startup = useExtensionUiStartupContext();
  const declaration = extension.manifest.ui;
  if (!declaration) return null;
  const incident = startup?.incident?.extensionId === extension.manifest.id
    ? startup.incident
    : null;
  const diagnosticAt = latestUiDiagnosticAt(
    extension.manifest.id,
    diagnostics,
    incident?.startedAt,
  );
  return (
    <section className="extud-root">
      <h3>{t("extensions.detail.uiTitle")}</h3>
      {declaration.mode === "advanced" && (
        <div className="extp-message extp-message-warning">
          <ShieldWarning size="var(--icon-lg)" />
          <span>{t("extensions.detail.uiAdvancedWarning")}</span>
        </div>
      )}
      <SettingsCard className="extp-lines">
        <InfoLine
          label={t("extensions.detail.uiLevel")}
          value={t(`extensions.uiModes.${declaration.mode}`)}
        />
        {declaration.mode === "advanced" && (
          <InfoLine
            label={t("extensions.detail.uiArtifact")}
            value={t(extension.uiArtifact
              ? "extensions.detail.uiArtifactReady"
              : "extensions.detail.uiArtifactMissing")}
            error={!extension.uiArtifact}
          />
        )}
      </SettingsCard>
      {diagnosticAt && (
        <div className="extud-diagnostic" role="alert">
          <span>{t("extensions.diagnostics.uiGeneric")}</span>
          <time dateTime={diagnosticAt}>
            {formatExtensionDate(diagnosticAt, i18n.language)}
          </time>
        </div>
      )}
      {declaration.mode === "standard" && (
        <CatalogState
          extensionId={extension.manifest.id}
          catalog={catalog}
          locale={i18n.resolvedLanguage ?? i18n.language}
          busy={busy}
          onRetry={onRetry}
        />
      )}
    </section>
  );
}

function latestUiDiagnosticAt(
  extensionId: string,
  diagnostics: ExtensionDiagnostic[],
  interruptedAt?: string,
): string | undefined {
  const candidates = diagnostics
    .filter((diagnostic) => diagnostic.extensionId === extensionId
      && UI_DIAGNOSTIC_CODES.includes(diagnostic.code as typeof UI_DIAGNOSTIC_CODES[number]))
    .map((diagnostic) => diagnostic.occurredAt);
  if (interruptedAt) candidates.push(interruptedAt);
  return candidates.reduce<string | undefined>((latest, candidate) => (
    !latest || Date.parse(candidate) > Date.parse(latest) ? candidate : latest
  ), undefined);
}

function CatalogState({ extensionId, catalog, locale, busy, onRetry }: {
  extensionId: string;
  catalog: ReturnType<typeof useOptionalStandardCatalog>;
  locale: string;
  busy: boolean;
  onRetry: () => void;
}) {
  const { t } = useTranslation();
  const state = catalog?.state;
  const entries = state?.snapshot?.contributions.filter(
    (entry) => entry.extensionId === extensionId,
  ) ?? [];
  if (!state || state.kind === "loading") {
    return <p className="extud-state" role="status">{t("extensions.detail.uiLoading")}</p>;
  }
  if (state.kind === "error") {
    return <CatalogError busy={busy} onRetry={onRetry} />;
  }
  return (
    <>
      {state.kind === "stale-error" && <CatalogError busy={busy} onRetry={onRetry} />}
      {entries.length === 0 ? (
        <p className="extud-state">{t("extensions.detail.uiEmpty")}</p>
      ) : (
        <SettingsCard className="extud-list">
          {entries.map((entry) => (
            <ContributionRow key={entry.contributionId} entry={entry} locale={locale} />
          ))}
        </SettingsCard>
      )}
    </>
  );
}

function CatalogError({ busy, onRetry }: { busy: boolean; onRetry: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="extud-state extud-error" role="alert">
      <span>{t("extensions.detail.uiError")}</span>
      <button type="button" className="btn btn-sm btn-secondary" disabled={busy} onClick={onRetry}>
        {t("extensions.actions.retry")}
      </button>
    </div>
  );
}

function ContributionRow({ entry, locale }: { entry: StandardCatalogEntry; locale: string }) {
  const { t } = useTranslation();
  const contribution = entry.contribution;
  const target = contribution.type === "theme"
    ? t("extensions.detail.uiTheme")
    : t(`extensions.uiPlacements.${contribution.placement.replaceAll(".", "_")}`);
  return (
    <div className="extud-row">
      <span>{localizedText(contribution.label, locale)}</span>
      <span>{target}</span>
    </div>
  );
}

function InfoLine({ label, value, error = false }: {
  label: string;
  value: string;
  error?: boolean;
}) {
  return (
    <div className="extp-info-line">
      <span>{label}</span>
      <span className={error ? "extud-error-text" : undefined}>{value}</span>
    </div>
  );
}
