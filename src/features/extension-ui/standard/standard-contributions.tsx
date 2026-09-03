import { useCallback, useState } from "react";
import { PanelSlot } from "@/components/layout/panel-slots";
import { EmptyState } from "@/components/ui/empty-state";
import { Tooltip } from "@/components/ui/tooltip";
import { useTranslation } from "react-i18next";
import { useOptionalStandardCatalog } from "./catalog-context";
import { StandardContributionBoundary } from "./contribution-boundary";
import { StandardIcon } from "./icon-registry";
import { localizedText } from "./localized-text";
import { useContributionAction } from "./use-contribution-action";
import { StandardViewRenderer } from "./view-renderer";
import type { SlotOccupant } from "../slot-types";
import type { StandardCatalogEntry, StandardContribution, StandardView } from "./types";
import "./standard-ui.css";

export function useStandardEntry(occupant: SlotOccupant | undefined): StandardCatalogEntry | undefined {
  const catalog = useOptionalStandardCatalog();
  return occupant?.source.kind === "extension"
    ? catalog?.entry(occupant.source.extensionId, occupant.source.contributionId)
    : undefined;
}

export function StandardTabContent({ entry }: { entry: StandardCatalogEntry }) {
  const { t } = useTranslation();
  const revision = useOptionalStandardCatalog()?.snapshot?.revision ?? 0;
  const contribution = asContribution(entry, "tab");
  const list = contribution.list
    ? <StandardViewRenderer key={`${revision}:list:${entry.contributionId}`} entry={entry} view={contribution.list} />
    : <EmptyState message={t("extensions.ui.emptyPanel")} />;
  return (
    <StandardContributionBoundary entry={entry}>
      <PanelSlot name="list">{list}</PanelSlot>
      <PanelSlot name="detail">
        <StandardViewRenderer key={`${revision}:detail:${entry.contributionId}`} entry={entry} view={contribution.detail} />
      </PanelSlot>
    </StandardContributionBoundary>
  );
}

export function StandardSettingsContent({ entry }: { entry: StandardCatalogEntry }) {
  const contribution = asContribution(entry, "settingsTab");
  const revision = useOptionalStandardCatalog()?.snapshot?.revision ?? 0;
  return (
    <StandardContributionBoundary entry={entry}>
      <StandardViewRenderer key={`${revision}:${entry.contributionId}`} entry={entry} view={contribution.detail} />
    </StandardContributionBoundary>
  );
}

export function StandardNavigationButton({
  entry,
  active,
  onSelect,
}: {
  entry: StandardCatalogEntry;
  active: boolean;
  onSelect: () => void;
}) {
  const contribution = asContribution(entry, "tab");
  const label = localizedText(contribution.label);
  return (
    <StandardContributionBoundary entry={entry}>
      <Tooltip label={label} placement="top">
        <button
          type="button"
          className="icon-btn lpf-btn"
          aria-label={label}
          aria-current={active ? "page" : undefined}
          data-nav-active={active ? "true" : undefined}
          onClick={onSelect}
        >
          <StandardIcon name={contribution.icon} />
        </button>
      </Tooltip>
    </StandardContributionBoundary>
  );
}

export function StandardPlacementAction({
  entry,
  surface,
}: {
  entry: StandardCatalogEntry;
  surface: "toolbar" | "composer";
}) {
  const revision = useOptionalStandardCatalog()?.snapshot?.revision ?? 0;
  return <PlacementAction key={`${revision}:${entry.contributionId}`} entry={entry} surface={surface} />;
}

function PlacementAction({
  entry,
  surface,
}: {
  entry: StandardCatalogEntry;
  surface: "toolbar" | "composer";
}) {
  const contribution = asContribution(entry, "action");
  const label = localizedText(contribution.label);
  const [view, setView] = useState<StandardView | null>(null);
  const payload = useCallback(() => ({ fields: {} }), []);
  const action = useContributionAction(entry, payload, setView);
  const button = (
    <button
      type="button"
      className={surface === "toolbar"
        ? "icon-btn toolbar-btn xui-toolbar-action"
        : "btn btn-sm btn-secondary xui-composer-action"}
      aria-label={label}
      aria-busy={action.busyAction !== null || undefined}
      disabled={action.busyAction !== null}
      onClick={() => void action.run(contribution.actionId)}
    >
      <StandardIcon name={contribution.icon} />
      {surface === "composer" && <span>{label}</span>}
    </button>
  );
  return (
    <StandardContributionBoundary entry={entry}>
      <span className={`xui-placement-action xui-placement-action-${surface}`}>
        {surface === "toolbar" ? <Tooltip label={label}>{button}</Tooltip> : button}
        {view && (
          <div className="xui-action-result relief elev-float">
            <StandardViewRenderer key={entry.contributionId} entry={entry} view={view} />
          </div>
        )}
      </span>
    </StandardContributionBoundary>
  );
}

function asContribution<Type extends StandardContribution["type"]>(
  entry: StandardCatalogEntry,
  type: Type,
): Extract<StandardContribution, { type: Type }> {
  if (entry.contribution.type !== type) throw new Error("invalid_extension_ui_contribution");
  return entry.contribution as Extract<StandardContribution, { type: Type }>;
}
